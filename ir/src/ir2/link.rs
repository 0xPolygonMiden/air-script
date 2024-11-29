use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::{Rc, Weak};

pub struct Link<T>
where
    T: Sized,
{
    pub link: Rc<RefCell<T>>,
}

impl<T> Link<T> {
    pub fn new(data: T) -> Self {
        Self {
            link: Rc::new(RefCell::new(data)),
        }
    }
    pub fn borrow(&self) -> std::cell::Ref<T> {
        self.link.borrow()
    }
    pub fn borrow_mut(&self) -> std::cell::RefMut<T> {
        self.link.borrow_mut()
    }
}

impl<T: Debug> Debug for Link<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.link.borrow())
    }
}

impl<T> Clone for Link<T> {
    fn clone(&self) -> Self {
        Self {
            link: self.link.clone(),
        }
    }
}

impl<T> PartialEq for Link<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.link == other.link
    }
}

impl<T> Eq for Link<T> where T: Eq {}

impl<T> From<BackLink<T>> for Link<T> {
    fn from(value: BackLink<T>) -> Self {
        value.to_link().unwrap()
    }
}

impl<T> From<Rc<RefCell<T>>> for Link<T> {
    fn from(value: Rc<RefCell<T>>) -> Self {
        Self { link: value }
    }
}

pub struct BackLink<T> {
    pub link: Option<Weak<RefCell<T>>>,
}

impl<T> BackLink<T> {
    pub fn none() -> Self {
        Self { link: None }
    }
    pub fn to_link(&self) -> Option<Link<T>> {
        self.link.as_ref().map(|link| Link {
            link: link.upgrade().unwrap(),
        })
    }
}

impl<T> Debug for BackLink<T> {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl<T> Clone for BackLink<T> {
    fn clone(&self) -> Self {
        Self {
            link: self.link.clone(),
        }
    }
}

impl<T> PartialEq for BackLink<T> {
    /// Always returns true because the field should be ignored in comparisons.
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<T> Eq for BackLink<T> {}

impl<T> From<Link<T>> for BackLink<T> {
    fn from(parent: Link<T>) -> Self {
        Self {
            link: Some(Rc::downgrade(&parent.link)),
        }
    }
}

impl<T> From<Rc<RefCell<T>>> for BackLink<T> {
    fn from(parent: Rc<RefCell<T>>) -> Self {
        Self {
            link: Some(Rc::downgrade(&parent)),
        }
    }
}
