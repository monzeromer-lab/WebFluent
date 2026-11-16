//! The order a body's `derived` values are worked out in: each after what
//! it reads, so declaration order cannot change a type, and `T35` for two
//! that read each other with neither declared — a type that depends on
//! itself has nothing to start from.

use std::collections::HashMap;

use super::Expr;

/// What `e` reads: each bare name, and each `Owner.member` — but for a
/// name a lambda inside it binds, which is the lambda's own (a match
/// expression's `.failed(r) { r }` binds `r` so).
pub(super) fn reads(e: &Expr, out: &mut Vec<(Option<String>, String)>) {
    fn walk(e: &Expr, bound: &mut Vec<String>, out: &mut Vec<(Option<String>, String)>) {
        match e {
            Expr::Identifier(n) if !bound.contains(n) => out.push((None, n.clone())),
            Expr::PropertyAccess(base, member) | Expr::OptionalProperty(base, member) => {
                if let Expr::Identifier(owner) = base.as_ref()
                    && !bound.contains(owner)
                {
                    out.push((Some(owner.clone()), member.clone()));
                }
            }
            Expr::Lambda(params, body) => {
                let names: Vec<String> = crate::syntax::parser::lambda_params(params)
                    .iter()
                    .map(|(n, _)| (*n).to_string())
                    .collect();
                let mark = bound.len();
                bound.extend(names);
                walk(body, bound, out);
                bound.truncate(mark);
                return;
            }
            _ => {}
        }
        for child in e.children() {
            walk(child, bound, out);
        }
    }
    walk(e, &mut Vec::new(), out);
}

/// The groups of `deps`' nodes that read each other, in an order where
/// each group comes after every group it reads: Tarjan's strongly
/// connected components, which come out reversed-topologically — what is
/// read first.
pub(super) fn components(deps: &[Vec<usize>]) -> Vec<Vec<usize>> {
    struct Walk<'a> {
        deps: &'a [Vec<usize>],
        index: Vec<Option<usize>>,
        low: Vec<usize>,
        on_stack: Vec<bool>,
        stack: Vec<usize>,
        next: usize,
        out: Vec<Vec<usize>>,
    }
    impl Walk<'_> {
        fn visit(&mut self, v: usize) {
            self.index[v] = Some(self.next);
            self.low[v] = self.next;
            self.next += 1;
            self.stack.push(v);
            self.on_stack[v] = true;
            for &w in &self.deps[v] {
                match self.index[w] {
                    None => {
                        self.visit(w);
                        self.low[v] = self.low[v].min(self.low[w]);
                    }
                    Some(i) if self.on_stack[w] => self.low[v] = self.low[v].min(i),
                    Some(_) => {}
                }
            }
            if Some(self.low[v]) == self.index[v] {
                let mut group = Vec::new();
                while let Some(w) = self.stack.pop() {
                    self.on_stack[w] = false;
                    group.push(w);
                    if w == v {
                        break;
                    }
                }
                group.sort_unstable();
                self.out.push(group);
            }
        }
    }
    let n = deps.len();
    let mut walk = Walk {
        deps,
        index: vec![None; n],
        low: vec![0; n],
        on_stack: vec![false; n],
        stack: Vec::new(),
        next: 0,
        out: Vec::new(),
    };
    for v in 0..n {
        if walk.index[v].is_none() {
            walk.visit(v);
        }
    }
    walk.out
}

/// Whether a group reads itself: two or more members, or one that reads
/// itself.
pub(super) fn is_cycle(group: &[usize], deps: &[Vec<usize>]) -> bool {
    group.len() > 1 || group.iter().any(|&v| deps[v].contains(&v))
}

/// Each node's dependencies among `names`, by what `values` read bare.
pub(super) fn bare_deps(names: &[&str], values: &[&Expr]) -> Vec<Vec<usize>> {
    let at: HashMap<&str, usize> = names.iter().enumerate().map(|(i, n)| (*n, i)).collect();
    values
        .iter()
        .map(|v| {
            let mut read = Vec::new();
            reads(v, &mut read);
            let mut deps: Vec<usize> = read
                .iter()
                .filter(|(owner, _)| owner.is_none())
                .filter_map(|(_, n)| at.get(n.as_str()).copied())
                .collect();
            deps.sort_unstable();
            deps.dedup();
            deps
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_come_after_what_they_read() {
        // 0 reads 1, 1 reads 2; 3 and 4 read each other.
        let deps = vec![vec![1], vec![2], vec![], vec![4], vec![3]];
        let groups = components(&deps);
        let at = |v: usize| groups.iter().position(|g| g.contains(&v)).unwrap();
        assert!(at(2) < at(1) && at(1) < at(0));
        assert!(groups.contains(&vec![3, 4]));
        assert!(is_cycle(&[3, 4], &deps));
        assert!(!is_cycle(&[0], &deps));
        assert!(is_cycle(&[0], &[vec![0]]));
    }
}
