use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

fn serialize_struct(s: &synstructure::Structure) -> TokenStream2 {
    assert_eq!(s.variants().len(), 1, "structs must have one variant");

    let variant: &synstructure::VariantInfo = &s.variants()[0];
    let encode = variant.each(|binding| {
        quote! {
            if fuel_tx::io::Serialize::size(#binding) % fuel_tx::io::ALIGN > 0 {
                return ::core::result::Result::Err(fuel_tx::io::Error::WrongAlign)
            }
            fuel_tx::io::Serialize::encode(#binding, buffer)?;
        }
    });

    let encode_extra = variant.each(|binding| {
        quote! {
            fuel_tx::io::Serialize::encode_extra(#binding, buffer)?;
        }
    });

    s.gen_impl(quote! {
        gen impl fuel_tx::io::Serialize for @Self {
            fn encode<O: fuel_tx::io::Output + ?Sized>(&self, buffer: &mut O) -> ::core::result::Result<(), fuel_tx::io::Error> {
                match self {
                    #encode
                };
                match self {
                    #encode_extra
                };

                ::core::result::Result::Ok(())
            }
        }
    })
}

fn serialize_enum(s: &synstructure::Structure) -> TokenStream2 {
    assert!(!s.variants().is_empty(), "got invalid empty enum");
    let encode_body = s.variants().iter().enumerate().map(|(i, v)| {
        let pat = v.pat();
        let index = i as u8;
        let encode_iter = v.bindings().iter().map(|binding| {
            quote! {
                if fuel_tx::io::Serialize::size(#binding) % fuel_tx::io::ALIGN > 0 {
                    return ::core::result::Result::Err(fuel_tx::io::Error::WrongAlign)
                }
                fuel_tx::io::Serialize::encode(#binding, buffer)?;
            }
        });
        let encode_extra_iter = v.bindings().iter().map(|binding| {
            quote! {
                fuel_tx::io::Serialize::encode_extra(#binding, buffer)?;
            }
        });
        quote! {
            #pat => {
                { <::core::primitive::u8 as fuel_tx::io::Serialize>::encode(&#index, buffer)?; }
                #(
                    { #encode_iter }
                )*
                #(
                    { #encode_extra_iter }
                )*
            }
        }
    });
    s.gen_impl(quote! {
        gen impl fuel_tx::io::Serialize for @Self {
            fn encode<O: fuel_tx::io::Output + ?Sized>(&self, buffer: &mut O) -> ::core::result::Result<(), fuel_tx::io::Error> {
                match self {
                    #(
                        #encode_body
                    )*,
                    _ => return ::core::result::Result::Err(fuel_tx::io::Error::UnknownDiscriminant),
                };

                ::core::result::Result::Ok(())
            }
        }
    })
}

/// Derives `Serialize` trait for the given `struct` or `enum`.
pub fn serialize_derive(mut s: synstructure::Structure) -> TokenStream2 {
    s.add_bounds(synstructure::AddBounds::Fields)
        .underscore_const(true);
    match s.ast().data {
        syn::Data::Struct(_) => serialize_struct(&s),
        syn::Data::Enum(_) => serialize_enum(&s),
        _ => panic!("Can't derive `Serialize` for `union`s"),
    }
}
