//! Beyond the Book: conversion traits and the common std traits.
//! Run: cargo run -p generics_traits_lifetimes --example std_traits

use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::ops::{Add, Index, Mul, Neg};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Vec2 {
    x: f64,
    y: f64,
}

// ---- Display: how a value looks to people ----
impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

// ---- Operator traits: +, *, unary -, [] ----
impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, k: f64) -> Vec2 {
        Vec2 {
            x: self.x * k,
            y: self.y * k,
        }
    }
}
impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        self * -1.0
    }
}
impl Index<usize> for Vec2 {
    type Output = f64;
    fn index(&self, i: usize) -> &f64 {
        match i {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Vec2 index {i} out of range"),
        }
    }
}

// ---- From / Into: infallible conversions ----
// Implement From; you get Into for free (a blanket impl in std).
impl From<(f64, f64)> for Vec2 {
    fn from((x, y): (f64, f64)) -> Self {
        Vec2 { x, y }
    }
}

// ---- TryFrom / TryInto: conversions that can fail ----
#[derive(Debug)]
struct Percent(u8);

impl TryFrom<i32> for Percent {
    type Error = String;
    fn try_from(n: i32) -> Result<Self, Self::Error> {
        if (0..=100).contains(&n) {
            Ok(Percent(n as u8))
        } else {
            Err(format!("{n} is not a percentage"))
        }
    }
}

// ---- AsRef: "anything I can cheaply view as a &str" ----
fn shout<S: AsRef<str>>(s: S) -> String {
    s.as_ref().to_uppercase()
}

// ---- Ord, PartialOrd, Eq, Hash: ordering and keys ----
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
struct Version {
    major: u32, // derived Ord compares fields top to bottom
    minor: u32,
}

fn main() {
    let a = Vec2 { x: 1.0, y: 2.0 };
    let b: Vec2 = (3.0, 4.0).into(); // Into, thanks to our From
    println!("a + b = {}", a + b);
    println!("a * 3 = {}", a * 3.0);
    println!("-a = {}, a[1] = {}", -a, a[1]);
    println!("default: {}", Vec2::default());

    if let Ok(Percent(n)) = Percent::try_from(42) {
        println!("valid: {n}%");
    }
    let p: Result<Percent, _> = 250.try_into();
    println!("{p:?}");

    // AsRef accepts &str, String, &String...
    println!("{} {}", shout("hi"), shout(String::from("there")));

    // Borrow: a HashMap<String, _> can be searched with a &str, because
    // String: Borrow<str> and both hash the same way.
    let mut ages: HashMap<String, u32> = HashMap::new();
    ages.insert("Ama".to_string(), 9);
    println!("Ama: {:?}", ages.get("Ama"));
    let owned = String::from("x");
    let viewed: &str = owned.borrow();
    println!("borrowed view: {viewed}");

    let mut versions = vec![
        Version {
            major: 1,
            minor: 10,
        },
        Version { major: 1, minor: 2 },
        Version {
            major: 0,
            minor: 99,
        },
    ];
    versions.sort();
    println!("sorted: {versions:?}");
    println!("max: {:?}", versions.iter().max());
}
