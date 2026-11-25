use super::*;

#[derive(Clone)]
pub(super) struct Record {
    pub name: Name,
    pub fields: Vec<(Name, Name)>,
}

impl Compiler {
    pub(super) fn prepare_records(&mut self) -> Result<(), String> {
        let names: Vec<_> = self.records.keys().cloned().collect();
        for name in names {
            self.prepare_record(&name)?;
        }
        Ok(())
    }
    pub(super) fn prepare_record(&mut self, name: &str) -> Result<(), String> {
        if self.prepared_records.contains(name) {
            return Ok(());
        }
        let record = self.records[name].clone();
        let mut names = BTreeSet::new();
        for (name, _) in &record.fields {
            if !names.insert(&name.text) {
                return Err(name.span.error(format!("duplicate field '{}'", name.text)));
            }
        }
        if self.record_preparation_depth >= 64 {
            return Err(record
                .name
                .span
                .error("recursive struct specialization limit exceeded"));
        }
        self.record_preparation_depth += 1;
        let fields = record
            .fields
            .iter()
            .try_for_each(|(_, ty)| self.check_type(ty));
        self.record_preparation_depth -= 1;
        fields?;
        let native = self.types[&record.name.text].clone();
        let copy = format!("copy_{native}");
        let assign = format!("assign_{native}");
        let scope = self.aggregate_scope(&record.name.text);
        for (operation, params, result, cpp, code, derivation) in [
            (
                "__copy__",
                vec![record.name.text.clone()],
                record.name.text.clone(),
                copy.clone(),
                format!(
                    "Obj<{native}> {copy}(Obj<{native}> value, Loc loc) {{ return deep_copy(value,loc); }}"
                ),
                None,
            ),
            (
                "__assign__",
                vec![record.name.text.clone(); 2],
                "Unit".into(),
                assign.clone(),
                String::new(),
                Some("record_assign"),
            ),
        ] {
            if self.has_manual_operation(operation, &record.name.text, params.len()) {
                continue;
            }
            let index = self.functions.len();
            self.aggregate_operations.insert(index);
            if operation == "__copy__" {
                self.record_copy_functions
                    .insert(record.name.text.clone(), index);
                self.register_derived_copy(index);
            }
            if let Some(kind) = derivation {
                self.native_derivation_tasks
                    .insert(index, (kind.into(), record.name.text.clone()));
            }
            self.functions.push(Function {
                scope,
                name: Name {
                    constant: None,
                    text: operation.into(),
                    span: record.name.span.clone(),
                },
                params,
                result,
                cpp,
                code,
                body: None,
                origin: Origin::Generated,
                checked: None,
            });
        }
        self.prepared_records.insert(name.into());
        self.prepare_tuple_functions(&record)?;
        self.register_aggregate_comparisons(&record.name.text, false, &record.name.span)?;
        self.register_aggregate_hash(&record.name.text, false, &record.name.span)?;
        Ok(())
    }

    pub(super) fn record_forwards(&self) -> String {
        self.records
            .keys()
            .map(|name| format!("struct {};\n", self.types[name]))
            .collect()
    }

    pub(super) fn record_cpp(&self) -> String {
        let mut code = String::new();
        for (name, record) in &self.records {
            let native = &self.types[name];
            code.push_str(&format!("struct {native} {{\n"));
            for (index, (_, ty)) in record.fields.iter().enumerate() {
                code.push_str(&format!("Obj<{}> field_{index};\n", self.types[&ty.text]));
            }
            code.push_str("};\n");
        }
        for name in self.records.keys() {
            let native = &self.types[name];
            code.push_str(&format!("Obj<{native}> deep_copy(const Obj<{native}>& value, Loc loc = Loc{{\"<copy>\",0,0}});\n"));
        }
        code
    }

    pub(super) fn prepare_record_copies(&mut self) -> Result<(), String> {
        let records: Vec<_> = self
            .records
            .values()
            .filter(|r| {
                self.record_copy_functions
                    .get(&r.name.text)
                    .is_some_and(|id| self.native_operation_needed(*id))
                    && !self.record_copies.contains_key(&r.name.text)
            })
            .cloned()
            .collect();
        for record in records {
            let copy_index = self.record_copy_functions[&record.name.text];
            let code = self.render_derived_copy(
                "record_copy",
                &record.name.text,
                copy_index,
                &record.name.span,
            )?;
            self.record_copies.insert(record.name.text, code);
        }
        Ok(())
    }

    pub(super) fn record_copy_cpp(&self) -> String {
        self.record_copies.values().cloned().collect()
    }

    pub(super) fn field(&self, ty: &str, name: &Name) -> Result<(usize, String), String> {
        if ty == crate::ir::hir::DIVERGED {
            return Err(crate::ir::hir::DIVERGED.into());
        }
        if self.callables.contains_key(ty) {
            return Err(name.span.error("callable environment fields are private"));
        }
        let record = self
            .records
            .get(ty)
            .ok_or_else(|| name.span.error(format!("type '{ty}' has no fields")))?;
        record
            .fields
            .iter()
            .enumerate()
            .find(|(_, (n, _))| n.text == name.text)
            .map(|(index, (_, ty))| (index, ty.text.clone()))
            .ok_or_else(|| {
                name.span
                    .error(format!("unknown field '{}' on '{ty}'", name.text))
            })
    }

    pub(super) fn empty_record_constructor(
        &mut self,
        name: &str,
        span: &Span,
    ) -> Result<Option<Value>, String> {
        let ty = self.resolve_type_text(name, &BTreeSet::new());
        let base = tuples::type_parts(&ty).map_or(ty.as_str(), |(base, _)| base);
        let empty = self.records.get(&ty).is_some_and(|r| r.fields.is_empty())
            || self
                .generic_records
                .get(base)
                .is_some_and(|r| r.fields.is_empty());
        if !empty {
            return Ok(None);
        }
        self.check_type(&Name {
            constant: None,
            text: ty.clone(),
            span: span.clone(),
        })?;
        Ok(Some(self.emit(&ty, Rvalue::Aggregate(vec![]))))
    }

    pub(super) fn record_value(
        &mut self,
        name: &Name,
        fields: &[(Name, Expr)],
    ) -> Result<Value, String> {
        self.check_type(name)?;
        let record = self
            .records
            .get(&name.text)
            .cloned()
            .ok_or_else(|| name.span.error(format!("unknown struct '{}'", name.text)))?;
        let mut values = BTreeMap::new();
        for (field, expr) in fields {
            let (index, ty) = self.field(&name.text, field)?;
            if values.contains_key(&index) {
                return Err(field
                    .span
                    .error(format!("duplicate field '{}'", field.text)));
            }
            let value = self.expr(expr, Some(&ty))?;
            self.same(&ty, &value.ty, &expr.span)?;
            values.insert(index, value);
        }
        for (index, (field, _)) in record.fields.iter().enumerate() {
            if !values.contains_key(&index) {
                return Err(name
                    .span
                    .error(format!("missing field '{}' in '{}'", field.text, name.text)));
            }
        }
        Ok(self.emit(
            &name.text,
            Rvalue::Aggregate(values.into_values().collect()),
        ))
    }
}
