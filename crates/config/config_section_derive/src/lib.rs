use proc_macro::{TokenStream, TokenTree};

#[proc_macro_attribute]
pub fn config_section(args: TokenStream, item: TokenStream) -> TokenStream {
    let path = args.to_string(); // the string literal, quotes included
    if !path.starts_with('"') {
        panic!("#[config_section] needs a string literal path, e.g. #[config_section(\"tts.fish\")]");
    }

    // The identifier after the top-level `struct` keyword.
    let mut tokens = item.clone().into_iter();
    let mut name = None;
    while let Some(tok) = tokens.next() {
        if let TokenTree::Ident(id) = &tok {
            if id.to_string() == "struct" {
                if let Some(TokenTree::Ident(n)) = tokens.next() {
                    name = Some(n.to_string());
                }
                break;
            }
        }
    }
    let name = name.expect("#[config_section] only supports structs");

    format!(
        "#[derive(::config::serde::Serialize, ::config::serde::Deserialize)]
         #[serde(default, crate = \"::config::serde\")]
         {item}

         impl ::config::ConfigSection for {name} {{
             const PATH: &'static str = {path};
         }}

         impl ::config::Configurable for {name} {{
             fn load(config: &::config::Config) -> Self {{
                 match config.load_into::<Self>() {{
                     Ok(v) => v,
                     Err(e) => {{
                         eprintln!(\"config: {{e}}\");
                         Self::default()
                     }}
                 }}
             }}

             fn store(&self, config: &mut ::config::Config) {{
                 if let Err(e) = config.store(self) {{
                     eprintln!(\"config: {{e}}\");
                 }}
             }}
         }}"
    )
    .parse()
    .unwrap()
}