use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::{spanned::Spanned, Ident, Type};

use crate::expression::generate_var;

pub struct Context {
    scopes: Vec<Scope>,
    // Allows for global variable analysis
    scope_history: Vec<Scope>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            scopes: vec![Scope::default()],
            scope_history: Default::default(),
        }
    }
}

impl Context {
    pub fn push_variable(&mut self, name: Ident, ty: Option<Type>) {
        self.scopes
            .last_mut()
            .expect("Scopes must at least have root scope")
            .variables
            .push((name, ty));
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::default())
    }

    pub fn pop_scope(&mut self) {
        let scope = self.scopes.pop().expect("Can't pop root scope");
        self.scope_history.push(scope);
    }

    pub fn restore_scope(&mut self) {
        let scope = self.scope_history.pop();
        if let Some(scope) = scope {
            self.scopes.push(scope);
        }
    }

    pub fn current_scope(&self) -> &Scope {
        self.scopes
            .last()
            .expect("Scopes must at least have root scope")
    }

    pub fn variable_type(&self, name: &Ident) -> Option<Option<Type>> {
        // Walk through each scope backwards until we find the variable.
        self.scopes
            .iter()
            .rev()
            .flat_map(|scope| scope.variables.iter().rev())
            .find(|(ident, _)| name.to_string() == ident.to_string())
            .map(|(_, ty)| ty.clone())
    }

    pub fn extend(&mut self, vars: impl IntoIterator<Item = (Ident, Option<Type>)>) {
        self.scopes
            .last_mut()
            .expect("Scopes must at least have root scope")
            .variables
            .extend(vars)
    }
}

#[derive(Default)]
pub struct Scope {
    variables: Vec<(Ident, (Option<Type>))>,
}

impl Scope {
    pub fn generate_vars(&self) -> Vec<TokenStream> {
        self.variables
            .iter()
            .map(|(ident, ty)| {
                let mut span = ident.span();
                let var = generate_var(ident, ty, span.clone());
                quote_spanned! {span=>
                    let #ident = #var;
                }
            })
            .collect()
    }
}
