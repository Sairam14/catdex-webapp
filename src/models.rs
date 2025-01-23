use diesel::{Queryable, Insertable};
use serde::{Deserialize, Serialize};
use super::schema::cats;

#[derive(Queryable, Serialize, Deserialize)]
pub struct Cat {
    pub id: i32,
    pub name: String,
    pub image_path: String,
}

#[derive(Serialize)]
pub struct IndexTemplateData {
    pub project_name: String,
    pub cats: Vec<Cat>,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = cats)]
pub struct NewCat {
    pub name: String,
    pub image_path: String,
}       