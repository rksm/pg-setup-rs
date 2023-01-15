use crate::error::Result;

pub struct TableBuilder {
    table_name: String,
    columns: Vec<Column>,
}

impl TableBuilder {
    pub fn new(table_name: impl ToString) -> TableBuilder {
        TableBuilder {
            table_name: table_name.to_string(),
            columns: Vec::new(),
        }
    }

    pub fn add_column<F>(
        &mut self,
        name: impl ToString,
        col_type: impl ToString,
        col_configurator: F,
    ) where
        F: FnOnce(&mut Column),
    {
        let mut col = Column {
            name: name.to_string(),
            col_type: col_type.to_string(),
            constraints: Default::default(),
        };
        col_configurator(&mut col);
        self.columns.push(col);
    }

    pub fn to_sql(&self) -> Result<String> {
        let mut sql = String::new();
        sql.push_str("CREATE TABLE IF NOT EXISTS ");
        sql.push_str(&self.table_name);
        sql.push_str(" (");
        for (i, col) in self.columns.iter().enumerate() {
            if i > 0 {
                sql.push_str(", ");
            }
            sql.push_str(&col.to_sql()?);
        }
        sql.push_str(");");
        Ok(sql)
    }
}

pub struct Column {
    name: String,
    col_type: String,
    constraints: Vec<String>,
}

impl Column {
    pub fn add_constraint(&mut self, constraint: impl ToString) {
        self.constraints.push(constraint.to_string());
    }

    pub fn primary_key(&mut self) {
        self.add_constraint("PRIMARY KEY");
    }

    pub fn not_null(&mut self) {
        self.add_constraint("NOT NULL");
    }

    pub fn unique(&mut self) {
        self.add_constraint("UNIQUE");
    }

    pub fn default(&mut self, default: impl ToString) {
        self.add_constraint(format!("DEFAULT {}", default.to_string()));
    }

    pub fn check(&mut self, check: impl ToString) {
        self.add_constraint(format!("CHECK ({})", check.to_string()));
    }

    pub fn references(&mut self, table: impl ToString, column: impl ToString) {
        self.add_constraint(format!(
            "REFERENCES {} ({})",
            table.to_string(),
            column.to_string()
        ));
    }

    pub fn foreign_key(&mut self, table: impl ToString, column: impl ToString) {
        self.references(table, column);
        self.add_constraint("FOREIGN KEY");
    }

    fn to_sql(&self) -> Result<String> {
        Ok(self.to_string())
    }
}

impl std::fmt::Display for Column {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.name, self.col_type)?;
        for constraint in &self.constraints {
            write!(f, " {}", constraint)?;
        }
        Ok(())
    }
}
