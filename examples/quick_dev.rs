use little_planner::{print_plan, Expr, RelExpr};

fn main() {
    print_example("join1 (automatic push down)", join1);
    print_example("join2 (manual push down)", join2);
    print_example("join3 (automatic push down)", join3);
    print_example("join4 (multiple select)", join4);
    print_example("select above join", select_above_join);
    print_example(
        "select pushed through project",
        select_pushed_through_project,
    );
    print_example("project pushed through join", project_pushed_through_join)
}

fn print_example(name: &str, example: fn()) {
    println!(">>> {}", name);
    example();
    // println!("<<< {}", name);
    println!();
}
fn join1() {
    let left = RelExpr::scan("a".into(), vec![0, 1]);
    let right = RelExpr::scan("x".into(), vec![2, 3]);

    let join = left.join(
        right,
        vec![
            Expr::col_ref(0).eq(Expr::col_ref(2)),
            Expr::col_ref(1).eq(Expr::int(100)),
        ],
    );

    print_plan(&join);
}

fn join2() {
    let left = RelExpr::scan("a".into(), vec![0, 1]);
    let right = RelExpr::scan("x".into(), vec![2, 3]);

    let join = left
        .select(vec![Expr::col_ref(1).eq(Expr::int(100))])
        .join(right, vec![Expr::col_ref(0).eq(Expr::col_ref(2))]);

    print_plan(&join);
}

fn join3() {
    let left = RelExpr::scan("a".into(), vec![0, 1]);
    let right = RelExpr::scan("x".into(), vec![2, 3]);

    let join = left.join(
        right,
        vec![
            Expr::col_ref(0).eq(Expr::col_ref(2)),
            Expr::col_ref(1).eq(Expr::int(100)),
            Expr::col_ref(3).eq(Expr::int(100)),
        ],
    );
    print_plan(&join);
}

fn join4() {
    let left = RelExpr::scan("a".into(), vec![0, 1]);
    let right = RelExpr::scan("x".into(), vec![2, 3]);
    let join = left.join(
        right,
        vec![
            Expr::col_ref(0).eq(Expr::int(100)),
            Expr::col_ref(1).eq(Expr::int(200)),
        ],
    );
    print_plan(&join);
}

// TODO Selects above Joins should merge their predicate sets,
fn select_above_join() {
    let left = RelExpr::scan("a".into(), vec![0, 1]);
    let right = RelExpr::scan("x".into(), vec![2, 3]);

    let join = left
        .join(right, vec![Expr::col_ref(0).eq(Expr::col_ref(2))])
        .select(vec![Expr::col_ref(1).eq(Expr::int(100))]);

    print_plan(&join);
}
// TODO Select should get pushed through Project where possible,
fn select_pushed_through_project() {
    let left = RelExpr::scan("a".into(), vec![0, 1]);

    let project = left
        .project([0].into())
        .select(vec![Expr::col_ref(0).eq(Expr::int(100))]);

    print_plan(&project);
}
// TODO Project should get pushed through Join where possible.
fn project_pushed_through_join() {
    let left = RelExpr::scan("a".into(), vec![0, 1]);
    let right = RelExpr::scan("x".into(), vec![2, 3]);

    let project = left
        .join(right, vec![Expr::col_ref(0).eq(Expr::col_ref(2))])
        .project([0].into());

    print_plan(&project);
}
