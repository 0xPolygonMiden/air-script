use std::{
    borrow::Borrow,
    collections::HashMap,
    hash::Hash,
    ops::{Index, IndexMut},
};

pub type Env<K, V> = Box<HashMap<K, V>>;

#[derive(Clone)]
pub enum LexicalScope<K, V> {
    Empty,
    Root(Env<K, V>),
    Nested(Box<LexicalScope<K, V>>, Env<K, V>),
}
impl<K, V> Default for LexicalScope<K, V> {
    fn default() -> Self {
        Self::Empty
    }
}
impl<K, V> LexicalScope<K, V> {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Empty => true,
            Self::Root(_) => false,
            Self::Nested(parent, env) => env.is_empty() && parent.is_empty(),
        }
    }

    /// Enters a new, nested lexical scope
    pub fn enter(&mut self) {
        let moved = Box::new(core::mem::take(self));
        *self = Self::Nested(moved, Env::default());
    }

    /// Exits the current lexical scope
    pub fn exit(&mut self) {
        match self {
            Self::Empty => (),
            Self::Root(_env) => {
                *self = Self::Empty;
            }
            Self::Nested(ref mut parent, _) => {
                let moved = core::mem::take(parent.as_mut());
                *self = moved;
            }
        }
    }
}
impl<K, V> LexicalScope<K, V>
where
    K: Eq + Hash,
{
    /// Inserts a new binding in the current scope, returning a conflicting definition
    /// if one is present (i.e. the same name was already declared in the same (current) scope).
    ///
    /// NOTE: This does not return `Some` if a previous definition exists in an outer scope,
    /// the new definition will shadow that one, but is not considered in conflict with it.
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        match self {
            Self::Empty => {
                let mut env = Env::default();
                env.insert(k, v);
                *self = Self::Root(env);
                None
            }
            Self::Root(ref mut env) => env.insert(k, v),
            Self::Nested(_, ref mut env) => env.insert(k, v),
        }
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        match self {
            Self::Empty => None,
            Self::Root(ref env) => env.get(key),
            Self::Nested(ref parent, ref env) => env.get(key).or_else(|| parent.get(key)),
        }
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        match self {
            Self::Empty => None,
            Self::Root(ref mut env) => env.get_mut(key),
            Self::Nested(ref mut parent, ref mut env) => {
                env.get_mut(key).or_else(|| parent.get_mut(key))
            }
        }
    }

    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        match self {
            Self::Empty => None,
            Self::Root(ref env) => env.get_key_value(key),
            Self::Nested(ref parent, ref env) => {
                env.get_key_value(key).or_else(|| parent.get_key_value(key))
            }
        }
    }

    /// Gets the value of the key stored in this structure by `key`
    ///
    /// This is used in some cases where a field of the key contains useful metadata
    /// (such as source spans), but is not part of the eq/hash impl. This function
    /// allows you to obtain the actual key stored in the map.
    pub fn get_key<Q>(&self, key: &Q) -> Option<&K>
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        self.get_key_value(key).map(|(k, _)| k)
    }
}
impl<K, V, Q> Index<&Q> for LexicalScope<K, V>
where
    K: Eq + Hash + Borrow<Q>,
    Q: Eq + Hash + ?Sized,
{
    type Output = V;

    #[inline]
    fn index(&self, key: &Q) -> &Self::Output {
        self.get(key).unwrap()
    }
}
impl<K, V, Q> IndexMut<&Q> for LexicalScope<K, V>
where
    K: Eq + Hash + Borrow<Q>,
    Q: Eq + Hash + ?Sized,
{
    #[inline]
    fn index_mut(&mut self, key: &Q) -> &mut Self::Output {
        self.get_mut(key).unwrap()
    }
}
