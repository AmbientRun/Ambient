use std::ops::Deref;

use glam::{Mat4, Vec2, Vec3, Vec4};
use ordered_float::OrderedFloat;

#[derive(Clone, Copy, Debug)]
pub struct OrderedMat4(pub Mat4);
impl std::hash::Hash for OrderedMat4 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for i in 0..4 {
            let row = self.0.row(i);
            OrderedFloat(row.x).hash(state);
            OrderedFloat(row.y).hash(state);
            OrderedFloat(row.z).hash(state);
            OrderedFloat(row.w).hash(state);
        }
    }
}
impl PartialEq for OrderedMat4 {
    fn eq(&self, other: &Self) -> bool {
        for i in 0..4 {
            let a = self.0.row(i);
            let b = other.0.row(i);
            if OrderedFloat(a.x) != OrderedFloat(b.x) {
                return false;
            }
            if OrderedFloat(a.y) != OrderedFloat(b.y) {
                return false;
            }
            if OrderedFloat(a.z) != OrderedFloat(b.z) {
                return false;
            }
            if OrderedFloat(a.w) != OrderedFloat(b.w) {
                return false;
            }
        }
        true
    }
}
impl Eq for OrderedMat4 {}
impl Deref for OrderedMat4 {
    type Target = Mat4;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AsRef<Mat4> for OrderedMat4 {
    fn as_ref(&self) -> &Mat4 {
        &self.0
    }
}
impl From<Mat4> for OrderedMat4 {
    fn from(t: Mat4) -> OrderedMat4 {
        OrderedMat4(t)
    }
}
impl From<OrderedMat4> for Mat4 {
    fn from(t: OrderedMat4) -> Mat4 {
        t.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct OrderedVec4(pub Vec4);
impl std::hash::Hash for OrderedVec4 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        OrderedFloat(self.x).hash(state);
        OrderedFloat(self.y).hash(state);
        OrderedFloat(self.z).hash(state);
        OrderedFloat(self.w).hash(state);
    }
}
impl PartialEq for OrderedVec4 {
    fn eq(&self, other: &Self) -> bool {
        if OrderedFloat(self.x) != OrderedFloat(other.x) {
            return false;
        }
        if OrderedFloat(self.y) != OrderedFloat(other.y) {
            return false;
        }
        if OrderedFloat(self.z) != OrderedFloat(other.z) {
            return false;
        }
        if OrderedFloat(self.w) != OrderedFloat(other.w) {
            return false;
        }
        true
    }
}
impl Eq for OrderedVec4 {}
impl Deref for OrderedVec4 {
    type Target = Vec4;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AsRef<Vec4> for OrderedVec4 {
    fn as_ref(&self) -> &Vec4 {
        &self.0
    }
}
impl From<Vec4> for OrderedVec4 {
    fn from(t: Vec4) -> OrderedVec4 {
        OrderedVec4(t)
    }
}
impl From<OrderedVec4> for Vec4 {
    fn from(t: OrderedVec4) -> Vec4 {
        t.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct OrderedVec3(pub Vec3);
impl std::hash::Hash for OrderedVec3 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        OrderedFloat(self.x).hash(state);
        OrderedFloat(self.y).hash(state);
        OrderedFloat(self.z).hash(state);
    }
}
impl PartialEq for OrderedVec3 {
    fn eq(&self, other: &Self) -> bool {
        if OrderedFloat(self.x) != OrderedFloat(other.x) {
            return false;
        }
        if OrderedFloat(self.y) != OrderedFloat(other.y) {
            return false;
        }
        if OrderedFloat(self.z) != OrderedFloat(other.z) {
            return false;
        }
        true
    }
}
impl Eq for OrderedVec3 {}
impl Deref for OrderedVec3 {
    type Target = Vec3;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AsRef<Vec3> for OrderedVec3 {
    fn as_ref(&self) -> &Vec3 {
        &self.0
    }
}
impl From<Vec3> for OrderedVec3 {
    fn from(t: Vec3) -> OrderedVec3 {
        OrderedVec3(t)
    }
}
impl From<OrderedVec3> for Vec3 {
    fn from(t: OrderedVec3) -> Vec3 {
        t.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct OrderedVec2(pub Vec2);
impl std::hash::Hash for OrderedVec2 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        OrderedFloat(self.x).hash(state);
        OrderedFloat(self.y).hash(state);
    }
}
impl PartialEq for OrderedVec2 {
    fn eq(&self, other: &Self) -> bool {
        if OrderedFloat(self.x) != OrderedFloat(other.x) {
            return false;
        }
        if OrderedFloat(self.y) != OrderedFloat(other.y) {
            return false;
        }
        true
    }
}
impl Eq for OrderedVec2 {}
impl Deref for OrderedVec2 {
    type Target = Vec2;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AsRef<Vec2> for OrderedVec2 {
    fn as_ref(&self) -> &Vec2 {
        &self.0
    }
}
impl From<Vec2> for OrderedVec2 {
    fn from(t: Vec2) -> OrderedVec2 {
        OrderedVec2(t)
    }
}
impl From<OrderedVec2> for Vec2 {
    fn from(t: OrderedVec2) -> Vec2 {
        t.0
    }
}
