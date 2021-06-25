#[derive(Copy, Clone)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T
}

impl<T: Copy> std::convert::From<[T; 2]> for Vec2<T> {
    fn from(v: [T; 2]) -> Vec2<T> {
        Vec2::<T> {
            x: v[0],
            y: v[1]
        }
    }
}

trait Vec2Into<T> {
    fn into_type(self: Self) -> Vec2<T>;
}

impl<T, A> Vec2Into<A> for Vec2<T>
where A: std::convert::From<T> {
    fn into_type(self) -> Vec2<A> {
       Vec2::<A> {
            x: A::from(self.x),
            y: A::from(self.y),
       }
    }
}

impl<T> Vec2<T> {
    pub fn map<A, B>(self, f: B) -> Vec2<A> where B: Fn(T) -> A {
        Vec2::<A> {
            x: f(self.x),
            y: f(self.y)
        }
    }
    pub fn to_type<A: std::convert::From<T>>(self) -> Vec2<A> {
        <Vec2<T> as Vec2Into<A>>::into_type(self)
    }
}

impl<T> std::ops::Add<Vec2<T>> for Vec2<T>
where T: std::ops::Add {
    type Output = Vec2<T::Output>;
    fn add(self, other: Vec2<T>) -> Vec2<T::Output> {
        Vec2::<T::Output> {
            x: self.x + other.x,
            y: self.y + other.y
        }
    }
}

impl<T> std::ops::Add<T> for Vec2<T>
where T: std::ops::Add + Copy {
    type Output = Vec2<T::Output>;
    fn add(self, other: T) -> Vec2<T::Output> {
        Vec2::<T::Output> {
            x: self.x + other,
            y: self.y + other
        }
    }
}


impl<T> std::ops::Sub<Vec2<T>> for Vec2<T>
where T: std::ops::Sub {
    type Output = Vec2<T::Output>;
    fn sub(self, other: Vec2<T>) -> Vec2<T::Output> {
        Vec2::<T::Output> {
            x: self.x - other.x,
            y: self.y - other.y
        }
    }
}

impl<T> std::ops::Sub<T> for Vec2<T>
where T: std::ops::Sub + Copy {
    type Output = Vec2<T::Output>;
    fn sub(self, other: T) -> Vec2<T::Output> {
        Vec2::<T::Output> {
            x: self.x - other,
            y: self.y - other
        }
    }
}

impl<T> std::ops::AddAssign<Vec2<T>> for Vec2<T>
where T: std::ops::Add<Output=T> + Copy {
    fn add_assign(&mut self, other: Vec2<T>) {
        *self = self.clone() + other;
    }

}

impl<T> std::ops::AddAssign<T> for Vec2<T>
where T: std::ops::Add<Output=T> + Copy {
    fn add_assign(&mut self, other: T) {
        *self = self.clone() + other;
    }

}

impl<T> std::ops::SubAssign<Vec2<T>> for Vec2<T>
where T: std::ops::Sub<Output=T> + Copy {
    fn sub_assign(&mut self, other: Vec2<T>) {
        *self = self.clone() - other;
    }

}

impl<T> std::ops::SubAssign<T> for Vec2<T>
where T: std::ops::Sub<Output=T> + Copy {
    fn sub_assign(&mut self, other: T) {
        *self = self.clone() - other;
    }

}

pub struct BoundingBox<T> {
    start: Vec2<T>,
    end: Vec2<T>
}

impl<T> BoundingBox<T> {
    pub fn new(x0: T, y0: T, x1: T, y1: T) -> Self {
        BoundingBox::<T> {
            start: Vec2::<T> {
                x: x0,
                y: y0
            },
            end: Vec2::<T> {
                x: x1,
                y: y1
            }
        }
    }
}

impl<T: Copy> BoundingBox<T> where T: std::ops::Sub<Output=T> {
    pub fn width(&self) -> T {
        self.end.x - self.start.x
    }
    pub fn height(&self) -> T {
        self.end.y - self.start.y
    }
    pub fn dimensions(&self) -> Vec2<T> {
        Vec2::<T> {
            x: self.width(),
            y: self.height()
        }
    }
}
