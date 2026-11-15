//! A connection's handle, typed by what it declares it sends and receives:
//! `chat.send(Outgoing(…))`, `chat.messages`, `match chat { open { … } }`.

use super::{Checker, Expr, Type, expr_text};
use crate::syntax::parser::ast::{ConnectionDecl, ConnectionKind};

/// What a closed connection says: its code and its reason.
pub(super) fn closure_type() -> Type {
    Type::Shape(vec![
        ("code".to_string(), Type::Number),
        ("reason".to_string(), Type::String),
    ])
}

fn error_type() -> Type {
    Type::Host("Error".to_string())
}

impl Checker<'_, '_> {
    /// The handle a connection's declaration makes.
    pub(super) fn connection_type(&self, c: &ConnectionDecl) -> Type {
        let json = || Type::Enum(super::ty::JSON.to_string());
        let send = c
            .sends
            .as_ref()
            .map_or_else(json, |t| self.world.resolve(Type::from_ref(t)));
        let receive = c
            .receives
            .as_ref()
            .map_or_else(json, |t| self.world.resolve(Type::from_ref(t)));
        Type::Connection(c.kind, Box::new(send), Box::new(receive))
    }

    /// Every member a connection of `kind` has, and its type.
    fn connection_members(
        kind: ConnectionKind,
        send: &Type,
        receive: &Type,
    ) -> Vec<(&'static str, Type)> {
        let func = |params: Vec<Type>, ret: Type| Type::Func(params, Box::new(ret));
        let last = func(
            vec![Type::optional(Type::String)],
            Type::optional(receive.clone()),
        );
        let mut members = vec![
            ("state", Type::String),
            ("error", Type::optional(error_type())),
            ("close", func(Vec::new(), Type::Null)),
        ];
        match kind {
            ConnectionKind::Worker => {
                members.push(("closure", Type::optional(closure_type())));
                members.push(("pending", Type::Number));
                members.push(("last", last));
                members.push(("send", func(vec![send.clone()], Type::Null)));
                members.push(("restart", func(Vec::new(), Type::Null)));
                members.push((
                    "request",
                    func(
                        vec![
                            Type::String,
                            Type::optional(Type::Enum(super::ty::JSON.to_string())),
                            Type::optional(Type::Enum(super::ty::JSON.to_string())),
                        ],
                        Type::Promise(Box::new(Type::Enum(super::ty::JSON.to_string()))),
                    ),
                ));
            }
            ConnectionKind::Channel => {
                members.push(("closure", Type::optional(closure_type())));
                members.push(("last", last));
                members.push(("post", func(vec![send.clone()], Type::Null)));
            }
            ConnectionKind::Stream => {
                members.push(("messages", Type::list(receive.clone())));
                members.push(("last", last));
            }
            _ => {
                members.push(("closure", Type::optional(closure_type())));
                members.push(("messages", Type::list(receive.clone())));
                members.push(("last", last));
                members.push(("send", func(vec![send.clone()], Type::Null)));
                if kind == ConnectionKind::Peer {
                    members.push((
                        "signal",
                        func(vec![Type::Enum(super::ty::JSON.to_string())], Type::Null),
                    ));
                }
            }
        }
        members
    }

    /// A member of a connection's handle, read: `chat.messages`.
    pub(super) fn connection_member(
        &mut self,
        kind: ConnectionKind,
        send: &Type,
        receive: &Type,
        base: &Expr,
        member: &str,
    ) -> Type {
        let members = Self::connection_members(kind, send, receive);
        if let Some((_, t)) = members.iter().find(|(n, _)| *n == member) {
            return t.clone();
        }
        let known: Vec<&str> = members.iter().map(|(n, _)| *n).collect();
        self.error_naming(
            "T05",
            format!("`{}` has no member `{member}`", expr_text(base)),
            &format!(
                "It has {}",
                known
                    .iter()
                    .map(|k| format!("`{k}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            member,
            &known,
            true,
        );
        Type::Error
    }

    /// What a connection's `on name(…)` handler is given.
    pub(super) fn connection_event(c_receive: &Type, event: &str) -> Type {
        match event {
            "message" => c_receive.clone(),
            "close" => closure_type(),
            "error" | "restart" => error_type(),
            _ => Type::Null,
        }
    }
}
