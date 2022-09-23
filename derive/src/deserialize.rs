use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

fn deserialize_struct(s: &synstructure::Structure) -> TokenStream2 {
    assert_eq!(s.variants().len(), 1, "structs must have one variant");

    let variant: &synstructure::VariantInfo = &s.variants()[0];
    let decode_main = variant.construct(|field, _| {
        let ty = &field.ty;
        quote! {
            <#ty as fuel_tx::io::Deserialize>::decode(buffer)?
        }
    });

    let decode_extra = variant.each(|binding| {
        quote! {
            fuel_tx::io::Deserialize::decode_extra(#binding, buffer)?;
        }
    });

    s.gen_impl(quote! {
        gen impl fuel_tx::io::Deserialize for @Self {
            fn decode<I: fuel_tx::io::Input + ?Sized>(buffer: &mut I) -> ::core::result::Result<Self, fuel_tx::io::Error> {
                let mut object = #decode_main;

                match object {
                    #decode_extra,
                };

                ::core::result::Result::Ok(object)
            }
        }
    })
}

fn deserialize_enum(s: &synstructure::Structure) -> TokenStream2 {
    assert!(!s.variants().is_empty(), "got invalid empty enum");
    let decode = s
        .variants()
        .iter()
        .map(|variant| {
            let decode_main = variant.construct(|field, _| {
                let ty = &field.ty;
                quote! {
                    <#ty as fuel_tx::io::Deserialize>::decode(buffer)?
                }
            });

            let decode_extra = variant.each(|binding| {
                quote! {
                    fuel_tx::io::Deserialize::decode_extra(#binding, buffer)?;
                }
            });

            quote! {
                {
                    let mut object = #decode_main;

                    match object {
                        #decode_extra,
                        // It is not possible, because we created `object` on previous iteration.
                        _ => panic!("unexpected variant of the enum"),
                    };

                    ::core::result::Result::Ok(object)
                }
            }
        })
        .enumerate()
        .fold(quote! {}, |acc, (i, v)| {
            let index = i as u8;
            quote! {
                #acc
                #index => #v,
            }
        });

    s.gen_impl(quote! {
        gen impl fuel_tx::io::Deserialize for @Self {
            fn decode<I: fuel_tx::io::Input + ?Sized>(buffer: &mut I) -> ::core::result::Result<Self, fuel_tx::io::Error> {
                match <::core::primitive::u8 as fuel_tx::io::Deserialize>::decode(buffer)? {
                    #decode
                    _ => return ::core::result::Result::Err(fuel_tx::io::Error::UnknownDiscriminant),
                }
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
