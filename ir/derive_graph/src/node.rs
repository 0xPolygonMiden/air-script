extern crate proc_macro;
use quote::{format_ident, quote};
use syn::{DeriveInput, Token};

pub fn impl_node_wrapper(input: &DeriveInput) -> proc_macro2::TokenStream {
    let ty = &input.ident;
    let name = format_ident!("{}", ty.to_string().to_lowercase());
    let fields = extract_struct_fields(input);
    let (node_field_name, field_names) = extract_field_names(&fields);
    let new_signature = make_new_signature(&field_names);
    let getters = make_getters(&field_names);
    let impls = quote! {
        use crate::ir::{BackLink, IsChild, Link, MiddleNode, NodeType, IsParent};
        use std::fmt::Debug;
        impl #ty {
            pub fn new(#(#new_signature)*) -> Self {
                Self {
                    #node_field_name: Node::new(
                        parent,
                        Link::new(vec![#(#field_names),*])
                    ),
                }
            }
            #(#getters)*
        }
        impl IsParent for #ty {
            fn get_children(&self) -> Link<Vec<Link<NodeType>>> {
                self.#node_field_name.get_children()
            }
        }
        impl IsChild for #ty {
            fn get_parent(&self) -> BackLink<NodeType> {
                self.#node_field_name.get_parent()
            }
            fn set_parent(&mut self, parent: Link<NodeType>) {
                self.#node_field_name.set_parent(parent);
            }
        }
        impl Debug for #ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}{:?}", stringify!(#ty), &self.#node_field_name)
            }
        }
        impl From<#ty> for Link<NodeType> {
            fn from(#name: #ty) -> Link<NodeType> {
                Link::new(NodeType::MiddleNode(MiddleNode::#ty(#name)))
            }
        }
    };
    impls
}

fn extract_struct_fields(input: &DeriveInput) -> Vec<&syn::Field> {
    match &input.data {
        syn::Data::Struct(data) => data.fields.iter().collect(),
        _ => panic!("NodeWrapper only supports structs"),
    }
}

fn extract_field_names(fields: &[&syn::Field]) -> (proc_macro2::Ident, Vec<proc_macro2::Ident>) {
    let node_field = fields
        .iter()
        .find(|field| field.attrs.iter().any(|attr| attr.path().is_ident("node")))
        .expect("NodeWrapper requires a node field");
    let node_field_name = node_field.ident.clone().unwrap();
    let field_names = node_field
        .attrs
        .iter()
        .find_map(|attr| {
            if attr.path().is_ident("node") {
                let args = attr
                    .parse_args_with(|input: syn::parse::ParseStream| {
                        syn::punctuated::Punctuated::<syn::Ident, Token![,]>::parse_terminated(
                            input,
                        )
                    })
                    .expect("Node field must have a list of field names")
                    .into_iter()
                    .collect::<Vec<_>>();
                Some(args)
            } else {
                None
            }
        })
        .expect("Node field must have a list of field names");
    (node_field_name, field_names)
}

fn make_new_signature(field_names: &[proc_macro2::Ident]) -> Vec<proc_macro2::TokenStream> {
    let mut signature = vec![quote! { parent: BackLink<NodeType> }];
    signature.extend(field_names.iter().map(|field_name| {
        quote! {
            ,
            #field_name: Link<NodeType>
        }
    }));
    signature
}

fn make_getters(field_names: &[proc_macro2::Ident]) -> Vec<proc_macro2::TokenStream> {
    field_names
        .iter()
        .enumerate()
        .map(|(field_index, field)| {
            let index = syn::Index::from(field_index);
            quote! {
                pub fn #field(&self) -> Link<NodeType> {
                    self.get_children().borrow()[#index].clone()
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use syn::parse2;

    #[test]
    fn test_derive_node_wrapper() {
        let input = quote! {
            #[derive(Node)]
            struct Test {
                #[node(lhs, rhs)]
                node_field: Node,
            }
        };
        let expected = quote! {
            use crate::ir::{BackLink, IsChild, Link, MiddleNode, NodeType, IsParent};
            use std::fmt::Debug;
            impl Test {
                pub fn new(parent: BackLink<NodeType>, lhs: Link<NodeType>, rhs: Link<NodeType>) -> Self {
                    Self {
                        node_field: Node::new(parent, Link::new(vec![lhs, rhs])),
                    }
                }
                pub fn lhs(&self) -> Link<NodeType> {
                    self.get_children().borrow()[0].clone()
                }
                pub fn rhs(&self) -> Link<NodeType> {
                    self.get_children().borrow()[1].clone()
                }
            }
            impl IsParent for Test {
                fn get_children(&self) -> Link<Vec<Link<NodeType>>> {
                    self.node_field.get_children()
                }
            }
            impl IsChild for Test {
                fn get_parent(&self) -> BackLink<NodeType> {
                    self.node_field.get_parent()
                }
                fn set_parent(&mut self, parent: Link<NodeType>) {
                    self.node_field.set_parent(parent);
                }
            }
            impl Debug for Test {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}{:?}", stringify!(Test), &self.node_field)
                }
            }
            impl From<Test> for Link<NodeType> {
                fn from(test: Test) -> Link<NodeType> {
                    Link::new(NodeType::MiddleNode(MiddleNode::Test(test)))
                }
            }
        };
        let ast = parse2(input).unwrap();
        let output = impl_node_wrapper(&ast);

        assert_eq!(output.to_string(), expected.to_string());
    }
}
