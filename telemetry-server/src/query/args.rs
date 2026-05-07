use std::{collections::{HashMap, HashSet}, str::FromStr};

use anyhow::anyhow;
use chrono::NaiveDateTime;

pub struct QueryArgs<'a> {
    data: HashMap<&'a str, &'a str>,
}

impl<'a> QueryArgs<'a> {
    pub fn new(query: &'a str) -> anyhow::Result<Self> {
        let mut data = HashMap::new();

        for pair in query.split('&') {
            let (k, v) = pair.split_once('=').ok_or(anyhow!("Missing \"=\" in query"))?;

            if k.is_empty() {
                return Err(anyhow!("Empty key"));
            }

            if v.is_empty() {
                return Err(anyhow!("Empty value"));
            }

            data.insert(k, v);
        }

        Ok(Self { data })
    }

    pub fn get_datetime(&self, name: &str) -> anyhow::Result<Option<&str>> {
        let err = anyhow!("Invalid value for: {name}");

        let Some(value) = self.data.get(name) else {
            return Ok(None);
        };
        
        let formats = [
            "%Y-%m-%d %H:%M:%S%.f",
            "%Y-%m-%d %H:%M:%S",
        ];

        for fmt in formats {
            if let Ok(_) = NaiveDateTime::parse_from_str(value, fmt) {
                return Ok(Some(value));
            }
        }

        Err(err)
    }

    pub fn get_int<T: FromStr>(&self, name: &str) -> anyhow::Result<Option<T>> {
        let err = anyhow!("Invalid value for: {name}");

        let Some(value) = self.data.get(name) else {
            return Ok(None);
        };

        match value.parse::<T>().map_err(|_| err) {
            Ok(v) => Ok(Some(v)),
            Err(e) => Err(e),
        }
    }

    pub fn get_csv(&self, name: &str) -> anyhow::Result<HashSet<String>> {
        let mut values = HashSet::new();
        for value in name.split(',') {
            let value = value.to_string();

            if values.contains(&value) {
                anyhow::bail!("Duplicate entry: {value}");
            }

            values.insert(value.to_string());
        }

        return Ok(values);
    }
}