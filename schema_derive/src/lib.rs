use proc_macro::TokenStream;
use proc_macro::quote
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Schema)]
pub fn schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let mut tables = vec![];

    if let syn::Data::Struct(data) = input.data {
        for field in data.fields {
            let fname = field.ident.unwrap().to_string();
            let table = capitalize(&fname);

            let ty = extract_ident(&field.ty);

            tables.push(quote! {
                schema.tables.push(<#ty as SchemaTable>::build(#table));
            });
        }
    }

    quote! {
        impl #name {
            pub fn schema() -> Schema {
                let mut schema = Schema {
                    tables: heapless::Vec::new(),
                };

                #(#tables)*

                schema
            }
        }
    }
    .into()
}

#[proc_macro_derive(SchemaTable)]
pub fn schema_table(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let mut cols = vec![];

    if let syn::Data::Struct(data) = input.data {
        for field in data.fields {
            let fname = field.ident.unwrap().to_string();
            let col_name = fname.clone();

            let ty = map_type(&field.ty);

            cols.push(quote! {
                table.columns.push(Column {
                    name: heapless::String::from(#col_name),
                    r#type: #ty,
                }).unwrap();
            });
        }
    }

    quote! {
        impl SchemaTable for #name {
            fn build(table_name: &str) -> Table {
                let mut table = Table {
                    name: heapless::String::new(),
                    columns: heapless::Vec::new(),
                };

                table.name.push_str(table_name).unwrap();

                #(#cols)*

                table
            }
        }
    }
    .into()
}

fn map_type(ty: &Type) -> proc_macro2::TokenStream {
    match ty {
        Type::Path(p) => {
            let seg = &p.path.segments.last().unwrap().ident;

            match seg.to_string().as_str() {
                "u16" => quote!(ColumnType::Unsigned16),
                "u32" => quote!(ColumnType::Unsigned32),
                "f32" => quote!(ColumnType::Float32),
                _ => quote!(ColumnType::String { max_len: 128 }),
            }
        }
        Type::Array(_) => quote!(ColumnType::String { max_len: 128 }),
        _ => quote!(ColumnType::String { max_len: 128 }),
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}