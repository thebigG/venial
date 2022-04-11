pub use crate::types::{
    Attribute, Declaration, Enum, EnumDiscriminant, EnumVariant, Function, GenericBound,
    GenericParam, GenericParams, GroupSpan, InlineGenericArgs, NamedField, Struct, StructFields,
    TupleField, TyExpr, Union, VisMarker, WhereClause, WhereClauseItem,
};
use proc_macro2::{Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

impl Declaration {
    /// Returns the [`GenericParams`], if any, of the declaration.
    ///
    /// For instance, this will return Some for `struct MyStruct<A, B, C> { ... }`
    /// and None for `enum MyEnum { ... }`.
    pub fn generic_params(&self) -> Option<&GenericParams> {
        match self {
            Declaration::Struct(struct_decl) => struct_decl.generic_params.as_ref(),
            Declaration::Enum(enum_decl) => enum_decl.generic_params.as_ref(),
            Declaration::Union(union_decl) => union_decl.generic_params.as_ref(),
            Declaration::Function(function_decl) => function_decl.generic_params.as_ref(),
        }
    }

    /// Returns the [`GenericParams`], if any, of the declaration.
    pub fn generic_params_mut(&mut self) -> Option<&mut GenericParams> {
        match self {
            Declaration::Struct(struct_decl) => struct_decl.generic_params.as_mut(),
            Declaration::Enum(enum_decl) => enum_decl.generic_params.as_mut(),
            Declaration::Union(union_decl) => union_decl.generic_params.as_mut(),
            Declaration::Function(function_decl) => function_decl.generic_params.as_mut(),
        }
    }

    /// Returns the [`Ident`] of the declaration.
    ///
    /// ```
    /// # use venial::parse_declaration;
    /// # use quote::quote;
    /// let struct_type = parse_declaration(quote!(
    ///     struct Hello(A, B);
    /// ));
    /// assert_eq!(struct_type.name().to_string(), "Hello");
    /// ```
    pub fn name(&self) -> Ident {
        match self {
            Declaration::Struct(struct_decl) => struct_decl.name.clone(),
            Declaration::Enum(enum_decl) => enum_decl.name.clone(),
            Declaration::Union(union_decl) => union_decl.name.clone(),
            Declaration::Function(function_decl) => function_decl.name.clone(),
        }
    }

    /// Returns the [`Struct`] variant of the enum if possible.
    pub fn as_struct(&self) -> Option<&Struct> {
        match self {
            Declaration::Struct(struct_decl) => Some(struct_decl),
            _ => None,
        }
    }

    /// Returns the [`Enum`] variant of the enum if possible.
    pub fn as_enum(&self) -> Option<&Enum> {
        match self {
            Declaration::Enum(enum_decl) => Some(enum_decl),
            _ => None,
        }
    }

    /// Returns the [`Union`] variant of the enum if possible.
    pub fn as_union(&self) -> Option<&Union> {
        match self {
            Declaration::Union(union_decl) => Some(union_decl),
            _ => None,
        }
    }

    /// Returns the [`Function`] variant of the enum if possible.
    pub fn as_function(&self) -> Option<&Function> {
        match self {
            Declaration::Function(function_decl) => Some(function_decl),
            _ => None,
        }
    }
}

impl Struct {
    /// Returns a collection of strings that can be used to exhaustively
    /// access the struct's fields.
    ///
    /// If the struct is a tuple struct, integer strings will be returned.
    ///
    /// ```
    /// # use venial::parse_declaration;
    /// # use quote::quote;
    /// let struct_type = parse_declaration(quote!(
    ///     struct Hello {
    ///         a: Foo,
    ///         b: Bar,
    ///     }
    /// ));
    /// let struct_type = struct_type.as_struct().unwrap();
    /// let field_names: Vec<_> = struct_type.field_names().into_iter().collect();
    /// assert_eq!(field_names, ["a", "b"]);
    /// ```
    ///
    /// ```
    /// # use venial::parse_declaration;
    /// # use quote::quote;
    /// let tuple_type = parse_declaration(quote!(
    ///     struct Hello(Foo, Bar);
    /// ));
    /// let tuple_type = tuple_type.as_struct().unwrap();
    /// let field_names: Vec<_> = tuple_type.field_names().into_iter().collect();
    /// assert_eq!(field_names, ["0", "1"]);
    /// ```
    pub fn field_names(&self) -> impl IntoIterator<Item = String> {
        match &self.fields {
            StructFields::Unit => Vec::new(),
            StructFields::Tuple(tuple_fields) => {
                let range = 0..tuple_fields.fields.len();
                range.map(|i| i.to_string()).collect()
            }
            StructFields::Named(named_fields) => named_fields
                .fields
                .items()
                .map(|field| field.name.to_string())
                .collect(),
        }
    }

    /// Returns a collection of tokens that can be used to exhaustively
    /// access the struct's fields.
    ///
    /// If the struct is a tuple struct, span-less integer literals will be returned.
    pub fn field_tokens(&self) -> impl IntoIterator<Item = TokenTree> {
        match &self.fields {
            StructFields::Unit => Vec::new(),
            StructFields::Tuple(tuple_fields) => {
                let range = 0..tuple_fields.fields.len();
                range.map(|i| Literal::usize_unsuffixed(i).into()).collect()
            }
            StructFields::Named(named_fields) => named_fields
                .fields
                .items()
                .map(|field| field.name.clone().into())
                .collect(),
        }
    }

    /// Returns a collection of references to the struct's field types.
    pub fn field_types(&self) -> impl IntoIterator<Item = &TyExpr> {
        match &self.fields {
            StructFields::Unit => Vec::new(),
            StructFields::Tuple(tuple_fields) => {
                tuple_fields.fields.items().map(|field| &field.ty).collect()
            }
            StructFields::Named(named_fields) => {
                named_fields.fields.items().map(|field| &field.ty).collect()
            }
        }
    }
}

impl Enum {
    /// Returns true if every single variant is empty.
    ///
    /// ```
    /// # use venial::parse_declaration;
    /// # use quote::quote;
    /// let enum_type = parse_declaration(quote!(
    ///     enum MyEnum { A, B, C, D }
    /// ));
    /// let enum_type = enum_type.as_enum().unwrap();
    /// assert!(enum_type.is_c_enum());
    /// ```
    pub fn is_c_enum(&self) -> bool {
        for variant in self.variants.items() {
            if !variant.is_empty_variant() {
                return false;
            }
        }
        true
    }
}

macro_rules! implement_type_setters {
    ($Kind:ident) => {
        #[allow(missing_docs)]
        // TODO - document
        impl $Kind {
            pub fn with_param(mut self, param: GenericParam) -> Self {
                let params = self.generic_params.take().unwrap_or_default();
                let params = params.with_param(param);
                self.generic_params = Some(params);
                self
            }

            pub fn with_where_item(mut self, item: WhereClauseItem) -> Self {
                if let Some(where_clause) = self.where_clause {
                    self.where_clause = Some(where_clause.with_item(item));
                } else {
                    self.where_clause = Some(WhereClause::from_item(item));
                }
                self
            }

            pub fn get_lifetime_params(&self) -> impl Iterator<Item = &GenericParam> {
                let params: &[_] = if let Some(params) = self.generic_params.as_ref() {
                    &params.params
                } else {
                    &[]
                };

                params
                    .iter()
                    .map(|(param, _punct)| param)
                    .filter(|param| GenericParam::is_lifetime(param))
            }

            pub fn get_type_params(&self) -> impl Iterator<Item = &GenericParam> {
                let params: &[_] = if let Some(params) = self.generic_params.as_ref() {
                    &params.params
                } else {
                    &[]
                };

                params
                    .iter()
                    .map(|(param, _punct)| param)
                    .filter(|param| GenericParam::is_ty(param))
            }

            pub fn get_const_params(&self) -> impl Iterator<Item = &GenericParam> {
                let params: &[_] = if let Some(params) = self.generic_params.as_ref() {
                    &params.params
                } else {
                    &[]
                };

                params
                    .iter()
                    .map(|(param, _punct)| param)
                    .filter(|param| GenericParam::is_const(param))
            }

            pub fn get_inline_generic_args(&self) -> Option<InlineGenericArgs<'_>> {
                Some(self.generic_params.as_ref()?.as_inline_args())
            }

            pub fn create_derive_where_clause(&self, derived_trait: TokenStream) -> WhereClause {
                let mut where_clause = self.where_clause.clone().unwrap_or_default();

                for param in self.get_type_params() {
                    let item = WhereClauseItem {
                        left_side: vec![param.name.clone().into()],
                        bound: GenericBound {
                            tk_colon: Punct::new(':', Spacing::Alone),
                            tokens: derived_trait.clone().into_iter().collect(),
                        },
                    };

                    where_clause = where_clause.with_item(item);
                }

                where_clause
            }
        }
    };
}

implement_type_setters! { Struct }
implement_type_setters! { Enum }
implement_type_setters! { Union }

impl EnumVariant {
    /// Returns true if the variant doesn't store a type.
    pub fn is_empty_variant(&self) -> bool {
        matches!(self.contents, StructFields::Unit)
    }

    /// Returns Some if the variant is a wrapper around a single type.
    /// Returns None otherwise.
    pub fn get_single_type(&self) -> Option<&TupleField> {
        match &self.contents {
            StructFields::Tuple(fields) if fields.fields.len() == 1 => Some(&fields.fields[0].0),
            StructFields::Tuple(_fields) => None,
            StructFields::Unit => None,
            StructFields::Named(_) => None,
        }
    }
}

#[allow(missing_docs)]
// TODO - document
impl GenericParams {
    pub fn with_param(mut self, param: GenericParam) -> Self {
        if param.is_lifetime() {
            self.params.insert(0, param, None);
        } else {
            self.params.push(param, None);
        }
        self
    }

    pub fn as_inline_args(&self) -> InlineGenericArgs<'_> {
        InlineGenericArgs(self)
    }
}

impl GenericParam {
    /// Create new lifetime param from name.
    ///
    /// ```
    /// # use venial::GenericParam;
    /// GenericParam::lifetime("a")
    /// # ;
    /// ```
    pub fn lifetime(name: &str) -> Self {
        let lifetime_ident = Ident::new(name, Span::call_site());
        GenericParam {
            tk_prefix: Some(Punct::new('\'', Spacing::Joint).into()),
            name: lifetime_ident,
            bound: None,
        }
    }

    /// Create new lifetime param from name and bound.
    ///
    /// ```
    /// # use venial::GenericParam;
    /// # use quote::quote;
    /// GenericParam::bounded_lifetime("a", quote!(b + c).into_iter().collect())
    /// # ;
    /// ```
    pub fn bounded_lifetime(name: &str, bound: Vec<TokenTree>) -> Self {
        let lifetime_ident = Ident::new(name, Span::call_site());
        GenericParam {
            tk_prefix: Some(Punct::new('\'', Spacing::Alone).into()),
            name: lifetime_ident,
            bound: Some(GenericBound {
                tk_colon: Punct::new(':', Spacing::Alone),
                tokens: bound,
            }),
        }
    }

    /// Create new type param from name.
    ///
    /// ```
    /// # use venial::GenericParam;
    /// GenericParam::ty("T")
    /// # ;
    /// ```
    pub fn ty(name: &str) -> Self {
        let ty_ident = Ident::new(name, Span::call_site());
        GenericParam {
            tk_prefix: None,
            name: ty_ident,
            bound: None,
        }
    }

    /// Create new type param from name and bound.
    ///
    /// ```
    /// # use venial::GenericParam;
    /// # use quote::quote;
    /// GenericParam::bounded_ty("T", quote!(Debug + Eq).into_iter().collect())
    /// # ;
    /// ```
    pub fn bounded_ty(name: &str, bound: Vec<TokenTree>) -> Self {
        let ty_ident = Ident::new(name, Span::call_site());
        GenericParam {
            tk_prefix: None,
            name: ty_ident,
            bound: Some(GenericBound {
                tk_colon: Punct::new(':', Spacing::Alone),
                tokens: bound,
            }),
        }
    }

    /// Create new const param from name and type.
    ///
    /// ```
    /// # use venial::GenericParam;
    /// # use quote::quote;
    /// GenericParam::const_param("N", quote!(i32).into_iter().collect())
    /// # ;
    /// ```
    pub fn const_param(name: &str, ty: Vec<TokenTree>) -> Self {
        let lifetime_ident = Ident::new(name, Span::call_site());
        GenericParam {
            tk_prefix: Some(Ident::new("const", Span::call_site()).into()),
            name: lifetime_ident,
            bound: Some(GenericBound {
                tk_colon: Punct::new(':', Spacing::Alone),
                tokens: ty,
            }),
        }
    }

    /// Returns true if the generic param is a lifetime param.
    pub fn is_lifetime(&self) -> bool {
        match &self.tk_prefix {
            Some(TokenTree::Punct(punct)) if punct.as_char() == '\'' => true,
            _ => false,
        }
    }

    /// Returns true if the generic param is a type param.
    pub fn is_ty(&self) -> bool {
        #[allow(clippy::redundant_pattern_matching)]
        match &self.tk_prefix {
            Some(_) => false,
            None => true,
        }
    }

    /// Returns true if the generic param is a const param.
    pub fn is_const(&self) -> bool {
        match &self.tk_prefix {
            Some(TokenTree::Ident(ident)) if ident == "const" => true,
            _ => false,
        }
    }
}

impl WhereClause {
    /// Create where-clause with a single item.
    pub fn from_item(item: WhereClauseItem) -> Self {
        Self::default().with_item(item)
    }

    /// Builder method, add an item to the where-clause.
    pub fn with_item(mut self, item: WhereClauseItem) -> Self {
        self.items.push(item, None);
        self
    }
}

impl WhereClauseItem {
    /// Helper method to create a WhereClauseItem from a quote.
    ///
    /// # Panics
    ///
    /// Panics if given a token stream that isn't a valid where-clause item.
    pub fn parse(tokens: TokenStream) -> Self {
        let mut tokens = tokens.into_iter().peekable();

        let left_side = crate::parse::consume_stuff_until(&mut tokens, |token| match token {
            TokenTree::Punct(punct) if punct.as_char() == ':' => true,
            _ => false,
        });

        let colon = match tokens.next().unwrap() {
            TokenTree::Punct(punct) if punct.as_char() == ':' => punct,
            _ => panic!("cannot parse type"),
        };

        let bound_tokens = tokens.collect();

        WhereClauseItem {
            left_side,
            bound: GenericBound {
                tk_colon: colon,
                tokens: bound_tokens,
            },
        }
    }
}

impl GroupSpan {
    /// Create from proc_macro2 Group.
    pub fn new(group: &Group) -> Self {
        Self {
            span: group.span(),
            delimiter: group.delimiter(),
        }
    }
}
