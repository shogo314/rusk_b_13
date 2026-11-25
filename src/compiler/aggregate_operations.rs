//! Source templates and metadata for automatically derived aggregate operations.
use super::*;
#[derive(Clone)]
pub(super) struct Declaration {
    scope: scopes::ScopeId,
    name: String,
    parameters: Vec<Name>,
    arguments: Vec<Name>,
}
impl Compiler {
    pub(super) fn copy_value(&mut self, value: Value, span: &Span) -> Result<Value, String> {
        let ty = value.ty.clone();
        let origin = self.context.declaration_scope;
        self.context.declaration_scope = self.aggregate_scope(&ty);
        let result = self.call("__copy__", vec![value], None, span);
        self.context.declaration_scope = origin;
        let result = result?;
        self.same(&ty, &result.ty, span)?;
        Ok(result)
    }

    pub(super) fn register_aggregate_declarations(&mut self, sources: &[Source]) {
        for source in sources {
            for statement in &source.statements {
                let (name, parameters, arguments) = match statement {
                    Stmt::UserFunction(name, parameters, arguments, ..)
                    | Stmt::GenericExternFunction(name, parameters, arguments, ..) => {
                        (name, parameters.clone(), arguments)
                    }
                    Stmt::Function(name, arguments, ..) => (name, vec![], arguments),
                    _ => continue,
                };
                if !matches!(
                    name.text.as_str(),
                    "__copy__" | "__assign__" | "__eq__" | "__ne__" | "__hash__"
                ) {
                    continue;
                }
                self.aggregate_declarations.push(Declaration {
                    scope: self.context.declaration_scope,
                    name: name.text.clone(),
                    parameters,
                    arguments: arguments.iter().map(|(_, ty)| ty.clone()).collect(),
                });
            }
        }
    }

    pub(super) fn aggregate_scope(&self, ty: &str) -> scopes::ScopeId {
        let ty = access_types::underlying(ty);
        let base = tuples::type_parts(ty).map_or(ty, |(base, _)| base);
        self.type_declaration_scopes
            .get(base)
            .copied()
            .unwrap_or(scopes::ScopeId::ROOT)
    }

    pub(super) fn has_manual_operation(&self, name: &str, ty: &str, arity: usize) -> bool {
        self.aggregate_declarations.iter().any(|d| {
            let mut bindings = BTreeMap::new();
            d.scope == self.aggregate_scope(ty)
                && d.name == name
                && d.arguments.len() == arity
                && d.arguments
                    .iter()
                    .all(|arg| generics::unify(&arg.text, ty, &d.parameters, &mut bindings))
        })
    }

    pub(super) fn aggregate_candidate(&self, id: usize, arguments: &[String]) -> bool {
        // Automatic operations belong to their type's scope, but do not create
        // a lexical family hiding unrelated scalar/container operations.
        !self.aggregate_operations.contains(&id)
            || self.functions[id]
                .params
                .iter()
                .zip(arguments)
                .all(|(a, b)| access_types::same(a, b))
    }

    pub(super) fn register_aggregate_comparisons(
        &mut self,
        ty: &str,
        is_enum: bool,
        span: &Span,
    ) -> Result<(), String> {
        if !self.types.contains_key("Bool")
            || ty.starts_with("Tuple<")
            || ty.starts_with("__RuskCallable")
        {
            return Ok(());
        }
        for name in ["__eq__", "__ne__"] {
            if self.has_manual_operation(name, ty, 2) {
                continue;
            }
            let kind = if name == "__ne__" {
                "tuple_ne"
            } else if is_enum {
                "enum_eq"
            } else {
                "tuple_eq"
            };
            let id =
                self.native_function(name, vec![ty.into(); 2], "Bool", |_| String::new(), span)?;
            self.functions[id].scope = self.aggregate_scope(ty);
            self.aggregate_operations.insert(id);
            self.native_derivation_tasks
                .insert(id, (kind.into(), ty.into()));
        }
        Ok(())
    }

    pub(super) fn register_aggregate_hash(
        &mut self,
        ty: &str,
        is_enum: bool,
        span: &Span,
    ) -> Result<(), String> {
        if !self.types.contains_key("Int")
            || ty.starts_with("__RuskCallable")
            || self.has_manual_operation("__hash__", ty, 1)
            || self.has_manual_operation("__eq__", ty, 2)
        {
            return Ok(());
        }
        let id =
            self.native_function("__hash__", vec![ty.into()], "Int", |_| String::new(), span)?;
        self.functions[id].scope = self.aggregate_scope(ty);
        self.aggregate_operations.insert(id);
        self.native_derivation_tasks.insert(
            id,
            (
                if is_enum { "enum_hash" } else { "record_hash" }.into(),
                ty.into(),
            ),
        );
        Ok(())
    }

    pub(super) fn native_operation_needed(&self, index: usize) -> bool {
        self.used.contains(&index)
            || self
                .native_effect_queries
                .iter()
                .any(|(id, _)| *id == index)
    }

    pub(super) fn render_derived_copy(
        &mut self,
        kind: &str,
        ty: &str,
        index: usize,
        span: &Span,
    ) -> Result<String, String> {
        self.request_native_effect(Some(index), index, crate::analysis::effects::Query::Writes);
        let code = self.render_native_derivation(kind, ty, index, "deep_copy", span)?;
        let dependencies = self.native_dependencies(index).cloned().unwrap_or_default();
        self.set_derived_copy_dependencies(index, ty, dependencies)?;
        Ok(code)
    }

    pub(super) fn prepare_derived_functions(&mut self) -> Result<(), String> {
        let tasks = self
            .native_derivation_tasks
            .iter()
            .filter(|(id, _)| {
                self.native_operation_needed(**id) && !self.native_derived_functions.contains(id)
            })
            .map(|(&id, (kind, ty))| (id, kind.clone(), ty.clone()))
            .collect::<Vec<_>>();
        for (index, kind, ty) in tasks {
            let function = self.functions[index].clone();
            self.functions[index].code = self.render_native_derivation(
                &kind,
                &ty,
                index,
                &function.cpp,
                &function.name.span,
            )?;
        }
        Ok(())
    }

    fn render_native_derivation(
        &mut self,
        kind: &str,
        ty: &str,
        index: usize,
        cpp: &str,
        span: &Span,
    ) -> Result<String, String> {
        let definition = self
            .native_derivations
            .get(kind)
            .cloned()
            .ok_or_else(|| span.error(format!("missing standard {kind} derivation")))?;
        self.functions[index].origin = definition.origin;
        self.native_derived_functions.insert(index);
        let pattern = if !definition.arguments.is_empty() {
            &definition.arguments[0].1.text
        } else {
            &definition.result.as_ref().unwrap().text
        };
        let mut bindings = BTreeMap::new();
        if !generics::unify(pattern, ty, &definition.parameters, &mut bindings) {
            return Err(span.error(format!("standard {kind} derivation does not match '{ty}'")));
        }
        let code = self
            .render_native(
                &definition.foreign.as_ref().unwrap()["code"],
                &bindings,
                Some(cpp),
                // Derived children inherit the original request site. Otherwise a
                // deferred child reader loses the user's location at each level.
                span,
                true,
                Some(index),
            )
            .map_err(|error| {
                format!(
                    "{error}\nwhile expanding standard {kind} declared at {}:{}:{}",
                    definition.name.span.file,
                    definition.name.span.line,
                    definition.name.span.column,
                )
            })?;
        let mut properties = definition.foreign.as_ref().unwrap().clone();
        let params = self.functions[index].params.clone();
        let result = self.functions[index].result.clone();
        if let Some(contract) = self.declared_native_effects(
            &definition.name,
            &params,
            &result,
            &properties,
            definition.origin,
        )? {
            self.native_info_mut(index).contract = Some(contract);
        }
        for key in ["lower", "lower_owned", "lower_borrowed", "lower_rooted"] {
            if let Some(expression) = properties.get(key).cloned() {
                let expression =
                    self.render_native(&expression, &bindings, Some(cpp), span, true, Some(index))?;
                properties.insert(key.into(), expression);
            }
        }
        self.register_native_lowering(
            index,
            &definition.name,
            &params,
            &result,
            &properties,
            definition.origin,
        )?;
        Ok(code)
    }

    pub(super) fn register_derived_copy(&mut self, index: usize) {
        self.native_info_mut(index).lowering = Some(crate::native::lowering::Lowering {
            rooted: Some(crate::native::lowering::Template::parse("deep_copy($0,$loc)",1).expect("rooted aggregate copy expression")),
            owned: Some(crate::native::lowering::Template::parse(&format!("([&](){{if constexpr(rusk_writes_{index}){{auto owner=$0;return deep_copy(owner,$loc);}}else return deep_copy($0,$loc);}}())"),1).expect("derived aggregate copy expression")),
            ..Default::default()
        });
    }
    pub(super) fn set_derived_copy_dependencies(
        &mut self,
        index: usize,
        ty: &str,
        dependencies: impl IntoIterator<Item = usize>,
    ) -> Result<(), String> {
        self.native_info_mut(index).contract = Some(crate::native::effects::contract(
            "checked_read_fresh",
            &[ty.into()],
            ty,
            &self.ir_types(),
        )?);
        self.native_info_mut(index).dependencies = Some(dependencies.into_iter().collect());
        Ok(())
    }
}
