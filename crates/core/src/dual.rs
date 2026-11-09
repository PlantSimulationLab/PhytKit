//! Forward-mode automatic differentiation.


#[derive(Clone, Debug)]
pub struct Dual {
    pub v: f64,
    pub d: Vec<f64>,
}

pub fn d_const(v: f64, m: usize) -> Dual {
    Dual { v, d: vec![0.0; m] }
}
pub fn d_var(v: f64, i: usize, m: usize) -> Dual {
    let mut d = vec![0.0; m];
    d[i] = 1.0;
    Dual { v, d }
}
pub fn d_add(a: &Dual, b: &Dual) -> Dual {
    Dual { v: a.v + b.v, d: a.d.iter().zip(&b.d).map(|(x, y)| x + y).collect() }
}
pub fn d_sub(a: &Dual, b: &Dual) -> Dual {
    Dual { v: a.v - b.v, d: a.d.iter().zip(&b.d).map(|(x, y)| x - y).collect() }
}
pub fn d_mul(a: &Dual, b: &Dual) -> Dual {
    Dual { v: a.v * b.v, d: a.d.iter().zip(&b.d).map(|(x, y)| x * b.v + a.v * y).collect() }
}
pub fn d_div(a: &Dual, b: &Dual) -> Dual {
    let inv = 1.0 / b.v;
    let inv2 = inv * inv;
    Dual { v: a.v * inv, d: a.d.iter().zip(&b.d).map(|(x, y)| (x * b.v - a.v * y) * inv2).collect() }
}
pub fn d_exp(a: &Dual) -> Dual {
    let e = a.v.exp();
    Dual { v: e, d: a.d.iter().map(|x| e * x).collect() }
}
pub fn d_log(a: &Dual) -> Dual {
    let inv = 1.0 / a.v;
    Dual { v: a.v.ln(), d: a.d.iter().map(|x| x * inv).collect() }
}
pub fn d_add_s(a: &Dual, s: f64) -> Dual {
    Dual { v: a.v + s, d: a.d.clone() }
}
pub fn d_mul_s(a: &Dual, s: f64) -> Dual {
    Dual { v: a.v * s, d: a.d.iter().map(|x| x * s).collect() }
}
pub fn d_s_sub(s: f64, a: &Dual) -> Dual {
    Dual { v: s - a.v, d: a.d.iter().map(|x| -x).collect() }
}
pub fn d_s_div(s: f64, a: &Dual) -> Dual {
    let inv = 1.0 / a.v;
    let f = -s * inv * inv;
    Dual { v: s * inv, d: a.d.iter().map(|x| f * x).collect() }
}
