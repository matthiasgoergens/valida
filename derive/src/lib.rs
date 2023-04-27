#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{spanned::Spanned, Data, Field, Fields, Ident};

#[proc_macro_derive(Machine, attributes(instruction, chip))]
pub fn machine_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_machine(&ast)
}

fn impl_machine(ast: &syn::DeriveInput) -> TokenStream {
    match &ast.data {
        Data::Struct(struct_) => {
            let fields = match &struct_.fields {
                Fields::Named(named) => named.named.iter().collect(),
                Fields::Unnamed(unnamed) => unnamed.unnamed.iter().collect(),
                Fields::Unit => vec![],
            };
            impl_machine_given_fields(&ast.ident, &fields)
        }
        _ => panic!("Machine derive only supports structs"),
    }
}

fn impl_machine_given_fields(machine: &Ident, fields: &[&Field]) -> TokenStream {
    let instructions = fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| a.path.is_ident("instruction")))
        .copied()
        .collect::<Vec<_>>();
    let chips = fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| a.path.is_ident("chip")))
        .copied()
        .collect::<Vec<_>>();
    let mut out = TokenStream2::new();
    let out1 = impl_machine_given_instructions_and_chips(machine, &instructions, &chips);
    let out2 = impl_machine_chip_impl_given_chips(machine, &chips);
    out.extend(out1);
    out.extend(out2);
    out.into()
}

fn impl_machine_chip_impl_given_chips(machine: &Ident, chips: &[&Field]) -> TokenStream2 {
    let chip_impls = chips.iter().map(|chip| {
        let chip_ty = &chip.ty;
        let tokens = quote!(#chip_ty);
        let chip_impl_name = Ident::new(&format!("MachineWith{}", tokens.to_string()), chip.span());
        let chip_methods = chip_methods(machine, chip);
        quote! {
            impl #chip_impl_name for #machine {
                #chip_methods
            }
        }
    });
    quote! {
        #(#chip_impls)*
    }
}

fn impl_machine_given_instructions_and_chips(
    machine: &Ident,
    instructions: &[&Field],
    chips: &[&Field],
) -> TokenStream2 {
    let run = run_method(machine, instructions);
    let prove = prove_method();
    let verify = verify_method();
    quote! {
        impl Machine for #machine {
            type F = ::valida_machine::DefaultField;
            #run
            #prove
            #verify
        }
    }
}

fn chip_methods(machine: &Ident, chip: &Field) -> TokenStream2 {
    let mut methods = vec![];
    let chip_name = chip.ident.as_ref().unwrap();
    let chip_name_mut = Ident::new(&format!("{}_mut", chip_name), chip_name.span());
    let chip_type = &chip.ty;
    methods.push(quote! {
        fn #chip_name(&self) -> &#chip_type {
            &self.#chip_name
        }
        fn #chip_name_mut(&mut self) -> &mut #chip_type {
            &mut self.#chip_name
        }
    });
    quote! {
        #(#methods)*
    }
}

fn run_method(machine: &Ident, instructions: &[&Field]) -> TokenStream2 {
    let opcode_arms = instructions
        .iter()
        .map(|inst| {
            let ident = &inst.ident;
            let ty = &inst.ty;
            quote! {
                <#ty as Instruction<#machine>>::OPCODE => {
                    #ty::execute(self, ops);
                }
            }
        })
        .collect::<TokenStream2>();
    quote! {
        fn run(&mut self) {
            loop {
                let opcode: u32 = 0u32; // TODO
                let ops = Operands::default(); // TODO
                match opcode {
                    #opcode_arms
                    _ => todo!(),
                };
            }
        }
    }
}

fn prove_method() -> TokenStream2 {
    quote! { fn prove(&self) {} }
}

fn verify_method() -> TokenStream2 {
    quote! { fn verify() {} }
}
