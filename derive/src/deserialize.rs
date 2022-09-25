use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

fn deserialize_struct(s: &synstructure::Structure) -> TokenStream2 {
    assert_eq!(s.variants().len(), 1, "structs must have one variant");

    let variant: &synstructure::VariantInfo = &s.variants()[0];
    let decode_main = variant.construct(|field, _| {
        let ty = &field.ty;
        quote! {
            <#ty as fuel_tx::io::Deserialize>::decode_static(buffer)?
        }
    });

    let decode_dynamic = variant.each(|binding| {
        quote! {
            fuel_tx::io::Deserialize::decode_dynamic(#binding, buffer)?;
        }
    });

    s.gen_impl(quote! {
        gen impl fuel_tx::io::Deserialize for @Self {
            fn decode_static<I: fuel_tx::io::Input + ?Sized>(buffer: &mut I) -> ::core::result::Result<Self, fuel_tx::io::Error> {
                ::core::result::Result::Ok(#decode_main)
            }

            fn decode_dynamic<I: fuel_tx::io::Input + ?Sized>(&mut self, buffer: &mut I) -> ::core::result::Result<(), fuel_tx::io::Error> {
                match self {
                    #decode_dynamic,
                };
                ::core::result::Result::Ok(())
            }
        }
    })
}

fn deserialize_enum(s: &synstructure::Structure) -> TokenStream2 {
    assert!(!s.variants().is_empty(), "got invalid empty enum");
    let decode_static = s
        .variants()
        .iter()
        .map(|variant| {
            let decode_main = variant.construct(|field, _| {
                let ty = &field.ty;
                quote! {
                    <#ty as fuel_tx::io::Deserialize>::decode_static(buffer)?
                }
            });

            quote! {
                {
                    ::core::result::Result::Ok(#decode_main)
                }
            }
        })
        .enumerate()
        .fold(quote! {}, |acc, (i, v)| {
            let index = i as u64;
            quote! {
                #acc
                #index => #v,
            }
        });

    let decode_dynamic = s.variants().iter().map(|variant| {
        let decode_dynamic = variant.each(|binding| {
            quote! {
                fuel_tx::io::Deserialize::decode_dynamic(#binding, buffer)?;
            }
        });

        quote! {
            #decode_dynamic
        }
    });

    s.gen_impl(quote! {
        gen impl fuel_tx::io::Deserialize for @Self {
            fn decode_static<I: fuel_tx::io::Input + ?Sized>(buffer: &mut I) -> ::core::result::Result<Self, fuel_tx::io::Error> {
                match <::core::primitive::u64 as fuel_tx::io::Deserialize>::decode(buffer)? {
                    #decode_static
                    _ => return ::core::result::Result::Err(fuel_tx::io::Error::UnknownDiscriminant),
                }
            }

            fn decode_dynamic<I: fuel_tx::io::Input + ?Sized>(&mut self, buffer: &mut I) -> ::core::result::Result<(), fuel_tx::io::Error> {
                match self {
                    #(
                        #decode_dynamic
                    )*
                    _ => return ::core::result::Result::Err(fuel_tx::io::Error::UnknownDiscriminant),
                };

                ::core::result::Result::Ok(())
            }
        }
    })
}

/// Derives `Deserialize` trait for the given `struct` or `enum`.
pub fn deserialize_derive(mut s: synstructure::Structure) -> TokenStream2 {
    s.bind_with(|_| synstructure::BindStyle::RefMut)
        .add_bounds(synstructure::AddBounds::Fields)
        .underscore_const(true);
    match s.ast().data {
        syn::Data::Struct(_) => deserialize_struct(&s),
        syn::Data::Enum(_) => deserialize_enum(&s),
        _ => panic!("Can't derive `Deserialize` for `union`s"),
    }
}
