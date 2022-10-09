#![feature(generic_associated_types)]
#![feature(async_fn_in_trait)]
#![feature(allocator_api)]
use std::{future::Future, pin::Pin, time::Duration};

use smol::{block_on, Timer};

trait Lang {
    type Repr<T>;
    fn int(iv: i64) -> Self::Repr<i64>;
    fn add(l: Self::Repr<i64>, r: Self::Repr<i64>) -> Self::Repr<i64>;
    fn from(l: <Self as Lang2>::Repr<i64>) -> <Self as Lang>::Repr<i64> where Self: Lang2;
}


trait Lang2 {
    type Repr<T>;

    fn pow(n: Self::Repr<i64>, p: Self::Repr<i64>) -> Self::Repr<i64>;
    fn from(l: <Self as Lang>::Repr<i64>) -> <Self as Lang2>::Repr<i64> where Self: Lang;
}

struct Print;

impl Lang for Print {
    type Repr<T> = String;

    fn int(iv: i64) -> Self::Repr<i64> {
        iv.to_string()
    }

    fn add(l: String, r: String) -> Self::Repr<i64> {
       format!("{l} + {r}")
    }

    fn from(l: String) -> String where Self: Lang2 {
        l
    }
}
impl Lang2 for Print {
    type Repr<T> = String;

    fn pow(n: String, p: String) -> String {
        match p.contains(" ") {
            true => format!("{n}^({p})"),
            false => format!("{n}^{p}")
        }
    }

    fn from(l: <Self as Lang>::Repr<i64>) -> <Self as Lang2>::Repr<i64> where Self: Lang {
        l
    }
}

struct Calc;
impl Lang for Calc {
    type Repr<T> = i64;

    fn int(iv: i64) -> i64 {
        iv
    }

    fn add(l: i64, r: i64) -> i64 {
       l + r
    }

    fn from(l: i64) -> i64 {
        l
    }
}
impl Lang2 for Calc {
    type Repr<T> = i64;

    fn pow(n: Self::Repr<i64>, p: i64) -> Self::Repr<i64> {
        i64::pow(n, p.try_into().unwrap())
    }

    fn from(l: <Self as Lang>::Repr<i64>) -> <Self as Lang2>::Repr<i64> where Self: Lang {
        l
    }
    
}
struct BoxedCalc;

impl Lang for BoxedCalc {
    type Repr<T> = Box<i64>;

    fn int(iv: i64) -> Self::Repr<i64> {
        Box::new(iv)
    }

    fn add(l: Box<i64>, r: Box<i64>) -> Box<i64> {
        Box::new(*l + *r)
    }

    fn from(l: Box<i64>) -> Box<i64>  {
        l
    }
}
impl Lang2 for BoxedCalc {
    type Repr<T> = Box<i64>;

    fn pow(n: Self::Repr<i64>, p: Self::Repr<i64>) -> Self::Repr<i64> {
        Box::new(i64::pow(*n, *p as u32))
    }

    fn from(l: <Self as Lang>::Repr<i64>) -> <Self as Lang2>::Repr<i64> where Self: Lang {
        l
    }
}

struct LazyCalc;
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a, std::alloc::Global>>;
impl Lang for LazyCalc {
    type Repr<T> = BoxFuture<'static,i64>;

    fn int(iv: i64) -> BoxFuture<'static,i64> {
        Box::pin(async move { iv })
    }

    fn add(l: Self::Repr<i64>, r: Self::Repr<i64>) -> BoxFuture<'static,i64>{
        Box::pin(async { 
            let l = l.await;
            let r = r.await;
            l + r
         })
    }

    fn from(l: <Self as Lang2>::Repr<i64>) -> <Self as Lang>::Repr<i64> where Self: Lang2 {
        l
    }

   

    
}

impl Lang2 for LazyCalc {
    type Repr<T> = BoxFuture<'static,i64>;

    fn pow(n: Self::Repr<i64>, p: Self::Repr<i64>) -> Self::Repr<i64> {
        
        Box::pin(async move {
            Timer::after(Duration::from_millis(1000)).await;
            let l = n.await;
            let r = p.await;
            let z = i64::pow(l, r as u32); 
            z 
        })
    }

    fn from(l: <Self as Lang>::Repr<i64>) -> <Self as Lang2>::Repr<i64> where Self: Lang {
        l
    }

}
fn main() {
    fn prog<L>() -> <L as Lang2>::Repr<i64> where L: Lang + Lang2, {
        L::pow(<L as Lang2>::from(L::int(2)),<L as Lang2>::from(L::add(L::int(4), L::int(20))))
    }
    println!("{}", prog::<Print>());
    println!("{}", prog::<Calc>());
    println!("{}", prog::<BoxedCalc>());
    block_on(async {
        println!("{}", prog::<LazyCalc>().await);
    });
    
}