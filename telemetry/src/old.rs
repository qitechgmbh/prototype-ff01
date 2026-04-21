use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    Zip,
    Folder,
    List,
}

#[derive(Debug, Deserialize)]
pub struct Schema {
    pub r#type: NodeType,
    pub items: Vec<Node>,
    pub templates: Option<HashMap<String, Template>>,
}

#[derive(Debug, Deserialize)]
pub struct Node {
    pub r#type: NodeType,
    pub name: Option<String>,
    pub items: Option<Items>,
    #[serde(rename = "$ref")]
    pub reference: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Items {
    List(ListNode),
    Nodes(Vec<Node>),
}

#[derive(Debug, Deserialize)]
pub struct ListNode {
    pub r#type: String, // "folder"
    pub of: Box<Node>,
}

#[derive(Debug, Deserialize)]
pub struct Template {
    #[serde(flatten)]
    pub entry: HashMap<String, CsvSpec>,
}

#[derive(Debug, Deserialize)]
pub struct CsvSpec {
    pub r#type: String,
    pub columns: HashMap<String, String>,
}

fn load_schema(path: &str) -> Schema {
    use std::fs;

    let data = fs::read_to_string(path).expect("Failed to read file");
    serde_yaml::from_str(&data).expect("Invalid YAML")
}

fn main() {
    let schema = load_schema("/home/entity/work/qitech/prototype-ff01/schema/archive.yaml");

    

    println!("{:#?}", schema);
}