use std::collections::HashSet;

pub enum Expr {
    ColRef { id: usize },
    Int { val: i64 },
    Eq { left: Box<Expr>, right: Box<Expr> },
}

impl Expr {
    pub fn col_ref(id: usize) -> Self {
        Expr::ColRef { id }
    }

    pub fn int(val: i64) -> Self {
        Expr::Int { val }
    }
    pub fn eq(self, other: Self) -> Self {
        Expr::Eq {
            left: Box::new(self),
            right: Box::new(other),
        }
    }
}
impl Expr {
    pub fn free(&self) -> HashSet<usize> {
        match self {
            Expr::ColRef { id } => {
                let mut set = HashSet::new();
                set.insert(*id);
                set
            }
            Expr::Int { .. } => HashSet::new(),
            Expr::Eq { left, right } => {
                let mut set = left.free();
                set.extend(right.free());
                set
            }
        }
    }
    pub fn bound_by(&self, rel: &RelExpr) -> bool {
        self.free().is_subset(&rel.att())
    }
}

pub enum RelExpr {
    Scan {
        table_name: String,
        column_names: Vec<usize>,
    },
    Select {
        src: Box<RelExpr>,
        predicates: Vec<Expr>,
    },
    Join {
        left: Box<RelExpr>,
        right: Box<RelExpr>,
        predicates: Vec<Expr>,
    },
    Project {
        src: Box<RelExpr>,
        cols: HashSet<usize>,
    },
}

// TODO [ok] Selects above Joins should merge their predicate sets,
// TODO [ok] Select should get pushed through Project where possible,
// TODO Project should get pushed through Join where possible.

impl RelExpr {
    pub fn scan(table_name: String, column_names: Vec<usize>) -> Self {
        RelExpr::Scan {
            table_name,
            column_names,
        }
    }

    pub fn select(self, mut predicates: Vec<Expr>) -> Self {
        println!("select");
        if let RelExpr::Select {
            src,
            predicates: mut preds,
        } = self
        {
            preds.append(&mut predicates);
            return src.select(preds);
        }
        if let RelExpr::Join {
            left,
            right,
            predicates: mut preds,
        } = self
        {
            preds.append(&mut predicates);
            return RelExpr::Join {
                left,
                right,
                predicates: preds,
            };
        }
        if let RelExpr::Project { src, cols } = self {
            return src.select(predicates).project(cols);
        }
        RelExpr::Select {
            src: Box::new(self),
            predicates,
        }
    }

    pub fn join(self, other: Self, mut predicates: Vec<Expr>) -> Self {
        println!("join");
        for i in 0..predicates.len() {
            if predicates[i].bound_by(&self) {
                // We can push this predicate down
                let predicate = predicates.swap_remove(i);
                return self.select(vec![predicate]).join(other, predicates);
            }

            if predicates[i].bound_by(&other) {
                // We can push this predicate down
                let predicate = predicates.swap_remove(i);
                return self.join(other.select(vec![predicate]), predicates);
            }
        }
        RelExpr::Join {
            left: Box::new(self),
            right: Box::new(other),
            predicates,
        }
    }

    pub fn project(self, cols: HashSet<usize>) -> Self {
        println!("project");
        if let RelExpr::Join {
            left,
            right,
            predicates,
        } = self
        {
            if cols.is_subset(&left.att()) {
                // return left.project(cols).join(*right, predicates);
                return RelExpr::Join {
                    left: Box::new(RelExpr::Project { src: left, cols }),
                    right,
                    predicates,
                };
            }
            if cols.is_subset(&right.att()) {
                // return left.join(right.project(cols), predicates);
                return RelExpr::Join {
                    left,
                    right: Box::new(RelExpr::Project { src: right, cols }),
                    predicates,
                };
            }
            return RelExpr::Project {
                src: Box::new(RelExpr::Join {
                    left,
                    right,
                    predicates,
                }),
                cols,
            };
        }
        RelExpr::Project {
            src: Box::new(self),
            cols,
        }
    }
}

impl RelExpr {
    pub fn att(&self) -> HashSet<usize> {
        match self {
            RelExpr::Scan { column_names, .. } => column_names.iter().cloned().collect(),
            RelExpr::Select { src, .. } => src.att(),
            RelExpr::Join { left, right, .. } => {
                let mut set = left.att();
                set.extend(right.att());
                set
            }
            RelExpr::Project { cols, .. } => cols.clone(),
        }
    }
    pub fn format_plan(&self) -> String {
        match self {
            RelExpr::Scan {
                table_name,
                column_names,
            } => format!("scan({:?}, {:?})", table_name, column_names),
            RelExpr::Select { predicates, .. } => {
                format!("select({})", format_predicates(predicates))
            }
            RelExpr::Join { predicates, .. } => format!("join({})", format_predicates(predicates)),
            RelExpr::Project { cols, .. } => format!("project({:?})", cols),
        }
    }
}

pub fn print_plan(plan: &RelExpr) {
    print_plan_impl(plan, 0);
}

fn print_plan_impl(plan: &RelExpr, level: usize) {
    println!("{}-> {}", "  ".repeat(level), plan.format_plan());
    match plan {
        RelExpr::Select { src, .. } => print_plan_impl(src, level + 1),
        RelExpr::Join { left, right, .. } => {
            print_plan_impl(left, level + 1);
            print_plan_impl(right, level + 1);
        }
        RelExpr::Project { src, .. } => print_plan_impl(src, level + 1),
        _ => {}
    }
}

fn format_predicates(predicates: &[Expr]) -> String {
    predicates
        .iter()
        .map(format_expr)
        .collect::<Vec<String>>()
        .join(" && ")
}

fn format_expr(expr: &Expr) -> String {
    match expr {
        Expr::ColRef { id } => format!("@{}", id),
        Expr::Eq { left, right } => format!("{}={}", format_expr(left), format_expr(right)),
        Expr::Int { val } => val.to_string(),
    }
}
