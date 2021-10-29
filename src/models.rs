use super::schema::jobs;

#[derive(Queryable)]
pub struct Jobs {
    pub id: u64,
    pub script: String,
}

#[derive(Insertable)]
#[table_name = "jobs"]
pub struct NewJob<'a> {
    pub script: &'a str,
}