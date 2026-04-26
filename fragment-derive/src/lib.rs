use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_derive(Fragment, attributes(Fragment))]
pub fn derive_fragment_table(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let table_name = format_ident!("{}Table", name);

    let fields = match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(f) => f.named,
            _ => panic!("named fields only"),
        },
        _ => panic!("struct only"),
    };

    let field_names: Vec<_> = fields.iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect();

    let field_types: Vec<_> = fields.iter()
        .map(|f| &f.ty)
        .collect();

    // map types
    let column_types: Vec<_> = field_types.iter().map(|ty| {
        let s = quote!(#ty).to_string();

        match s.as_str() {
            "u8"  => quote!(ColumnType::U8),
            "u16" => quote!(ColumnType::U16),
            "u32" => quote!(ColumnType::U32),
            "u64" => quote!(ColumnType::U64),
            "f32" => quote!(ColumnType::F32),
            "f64" => quote!(ColumnType::F64),
            _ => panic!("unsupported type"),
        }
    }).collect();

    let expanded = quote! {
        #[derive(Debug, Clone, Default)]
        pub struct #table_name {
            timestamp: Vec<u64>,
            #(#field_names: Vec<#field_types>,)*
        }

        impl #table_name {
            pub fn new() -> #table_name {
                Default::default()
            }
        }

        impl Table for #table_name {
            const SCHEMA: TableSchema = TableSchema {
                name: stringify!(#name),
                columns: &[
                    #(
                        ColumnSchema {
                            name: stringify!(#field_names),
                            ty: #column_types,
                        }
                    ),*
                ],
            };

            type Item = #name;

            fn append(&mut self, ts: u64, item: Self::Item) {
                self.ts.push(ts);
                #(self.#field_names.push(item.#field_names);)*
            }
        }
    };

    expanded.into()
}

#[proc_macro]
pub fn import_schema(input: TokenStream) -> TokenStream {
    let path = input.to_string().replace('"', "");

    let text = fs::read_to_string(&path)
        .expect("schema file not found");

    let parsed = parse(&text);

    let expanded = quote! {
        #parsed
    };

    expanded.into()
}

fn parse(input: &str) -> proc_macro2::TokenStream {
    use quote::quote;

    let mut tables = Vec::new();

    let mut current_name = None;
    let mut columns = Vec::new();

    for line in input.lines().map(|l| l.trim()).filter(|l| !l.is_empty()) {
        if line.starts_with('[') && line.ends_with(']') {
            if let Some(name) = current_name.take() {
                tables.push((name, std::mem::take(&mut columns)));
            }

            current_name = Some(line[1..line.len()-1].to_string());
            continue;
        }

        if let Some((k, v)) = line.split_once('=') {
            let r#type = match v.trim() {
                "u8"  => "ColumnType::Unsigned8",
                "u16" => "ColumnType::Unsigned16",
                "u32" => "ColumnType::Unsigned32",
                "u64" => "ColumnType::Unsigned64",
                "i8"  => "ColumnType::Signed8",
                "i16" => "ColumnType::Signed16",
                "i32" => "ColumnType::Signed32",
                "i64" => "ColumnType::Signed64",
                "f32" => "ColumnType::Float32",
                "f64" => "ColumnType::Float64",
                _ => panic!("bad type"),
            };

            columns.push((k.trim().to_string(), r#type.to_string()));
        }
    }

    if let Some(name) = current_name.take() {
        tables.push((name, columns));
    }

    let table_tokens = tables.into_iter().map(|(name, cols)| {
        let col_tokens = cols.into_iter().map(|(n, ty)| {
            quote! {
                fragment::ColumnSchema {
                    name: #n,
                    r#type: #ty,
                }
            }
        });

        quote! {
            fragment::TableSchema {
                name: #name,
                columns: &[ #(#col_tokens),* ],
            }
        }
    });

    quote! {
        fragment::FragmentSchema {
            tables: &[
                #(#table_tokens),*
            ]
        }
    }
}