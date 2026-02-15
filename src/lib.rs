use wasm_bindgen::prelude::*;
use web_sys::console;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

const FAR_CLIPPING_PLANE: f64 = f64::INFINITY;
const NEAR_CLIPPING_PLANE: f64 = 1.0;
const BACKGROUND_COLOR: Color = Color {
    r: 50,
    g: 50,
    b: 50,
};

#[derive(Debug, Clone, Copy)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}
impl Color {
    fn serialise(&self) -> String {
        format!("rgb({},{},{})", self.r, self.g, self.b)
    }

    fn mul(&self, intensity: f64) -> Color {
        return Color {
            r: (f64::from(self.r) * intensity) as u8,
            g: (f64::from(self.g) * intensity) as u8,
            b: (f64::from(self.b) * intensity) as u8,
        };
    }
}

#[derive(Debug, Clone, Copy)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}
impl Vec3 {
    fn add(&self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    fn sub(&self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    fn mul(&self, scalar: f64) -> Vec3 {
        Vec3 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }

    fn dot(&self, other: Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn len(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    fn dist(&self, other: Vec3) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2))
            .sqrt()
    }

    fn reverse(&self) -> Vec3 {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

struct Canvas {
    ctx: CanvasRenderingContext2d,

    width: i32,
    height: i32,

    vp_width: i32,
    vp_height: i32,
}

impl Canvas {
    fn to_viewport(&self, x: i32, y: i32) -> Vec3 {
        let vx = x as f64 * self.vp_width as f64 / self.width as f64;
        let vy = y as f64 * self.vp_height as f64 / self.height as f64;
        return Vec3 {
            x: vx,
            y: vy,
            z: NEAR_CLIPPING_PLANE,
        };
    }

    fn put_pixel(&self, x: i32, y: i32, color: Color) {
        let sx = self.width / 2 + x;
        let sy = self.height / 2 - y;
        self.ctx.set_fill_style_str(&color.serialise());
        self.ctx.fill_rect(sx as f64, sy as f64, 1.0, 1.0)
    }
}

#[derive(Debug, Clone, Copy)]
struct Sphere {
    position: Vec3,
    radius: f64,
    color: Color,
    specular: f64,
}
impl Sphere {
    fn intersect(&self, o: Vec3, d: Vec3) -> (f64, f64) {
        let r = self.radius;
        let co = o.sub(self.position);

        let a = d.dot(d);
        let b = 2.0 * d.dot(co);
        let c = co.dot(co) - r * r;

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return (FAR_CLIPPING_PLANE, FAR_CLIPPING_PLANE);
        }

        let t1 = (-b + discriminant.sqrt()) / (2.0 * a);
        let t2 = (-b - discriminant.sqrt()) / (2.0 * a);

        return (t1, t2);
    }
}

struct LightAmbient {
    intensity: f64,
}
struct LightPoint {
    intensity: f64,
    position: Vec3,
}
struct LightDirectional {
    intensity: f64,
    direction: Vec3,
}

struct Camera {
    position: Vec3,
    orientation: f64,
}

struct Scene {
    camera: Camera,
    spheres: Vec<Sphere>,
    light_ambient: LightAmbient,
    light_directional: Vec<LightDirectional>,
    light_point: Vec<LightPoint>,
}

impl Scene {
    fn trace_ray(&self, direction: Vec3) -> Color {
        let mut closest_t = FAR_CLIPPING_PLANE;
        let mut closest_sphere: Option<&Sphere> = None;

        for sphere in &self.spheres {
            let (t1, t2) = sphere.intersect(self.camera.position, direction);

            if t1 >= NEAR_CLIPPING_PLANE && t1 <= FAR_CLIPPING_PLANE && t1 < closest_t {
                closest_t = t1;
                closest_sphere = Some(sphere);
            }
            if t2 >= NEAR_CLIPPING_PLANE && t2 <= FAR_CLIPPING_PLANE && t2 < closest_t {
                closest_t = t2;
                closest_sphere = Some(sphere);
            }
        }

        if closest_sphere.is_none() {
            return BACKGROUND_COLOR;
        }

        let p = self.camera.position.add(direction.mul(closest_t));
        let n = p.sub(closest_sphere.unwrap().position);
        let nn = n.mul(1.0 / n.len());

        let sphere = closest_sphere.unwrap();
        return sphere.color.mul(self.compute_lighting(
            p,
            nn,
            direction.reverse(),
            sphere.specular,
        ));
    }

    fn compute_lighting(&self, point: Vec3, normal: Vec3, v: Vec3, specular: f64) -> f64 {
        let mut i: f64 = 0.0;

        i += self.light_ambient.intensity;

        for light in &self.light_point {
            let l = light.position.sub(point);

            // diffuse
            let n_dot_l = normal.dot(l);
            if n_dot_l > 0.0 {
                i += light.intensity * n_dot_l / (normal.len() * l.len());
            }

            // specular
            if specular != -1.0 {
                let r = normal.mul(2.0).mul(n_dot_l).sub(l);
                let r_dot_v = r.dot(v);
                if r_dot_v > 0.0 {
                    i += light.intensity * (r_dot_v / (r.len() * v.len())).powf(specular);
                }
            }
        }

        for light in &self.light_directional {
            let l = light.direction;

            // diffuse
            let n_dot_l = normal.dot(l);
            if n_dot_l > 0.0 {
                i += light.intensity * n_dot_l / (normal.len() * l.len());
            }

            // specular
            if specular != -1.0 {
                let r = normal.mul(2.0).mul(n_dot_l).sub(l);
                let r_dot_v = r.dot(v);
                if r_dot_v > 0.0 {
                    i += light.intensity * (r_dot_v / (r.len() * v.len())).powf(specular);
                }
            }
        }

        return i;
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    let window = web_sys::window().expect("no global window");
    let document = window.document().expect("no document on window");
    let dpr = window.device_pixel_ratio();

    let canvas = document
        .create_element("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    document.body().unwrap().append_child(&canvas).unwrap();

    let css_w = canvas.client_width();
    let css_h = canvas.client_height();

    let width = css_w * dpr as i32;
    let height = css_h * dpr as i32;

    canvas.set_width(width as u32);
    canvas.set_height(height as u32);

    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();

    ctx.scale(dpr, dpr).unwrap();

    let c = Canvas {
        ctx,
        width,
        height,
        vp_width: 1,
        vp_height: 1,
    };

    let scene = Scene {
        camera: Camera {
            position: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            orientation: 0.0,
        },
        light_ambient: LightAmbient { intensity: 0.2 },
        light_point: vec![LightPoint {
            intensity: 0.6,
            position: Vec3 {
                x: 2.0,
                y: 1.0,
                z: 0.0,
            },
        }],
        light_directional: vec![LightDirectional {
            intensity: 0.2,
            direction: Vec3 {
                x: 1.0,
                y: 4.0,
                z: 4.0,
            },
        }],
        spheres: vec![
            Sphere {
                specular: 500.0,
                radius: 1.0,
                position: Vec3 {
                    x: 2.0,
                    y: 0.0,
                    z: 4.0,
                },
                color: Color { r: 255, g: 0, b: 0 },
            },
            Sphere {
                specular: 500.0,
                radius: 1.0,
                position: Vec3 {
                    x: 0.0,
                    y: -1.0,
                    z: 3.0,
                },
                color: Color { r: 0, g: 0, b: 255 },
            },
            Sphere {
                specular: 10.0,
                radius: 1.0,
                position: Vec3 {
                    x: -2.0,
                    y: 0.0,
                    z: 4.0,
                },
                color: Color { r: 0, g: 255, b: 0 },
            },
            Sphere {
                specular: 1000.0,
                radius: 5000.0,
                position: Vec3 {
                    x: 0.0,
                    y: -5001.0,
                    z: 0.0,
                },
                color: Color {
                    r: 255,
                    g: 255,
                    b: 0,
                },
            },
        ],
    };

    for x in -width / 2..width / 2 {
        for y in -height / 2..height / 2 {
            let point = c.to_viewport(x, y);
            let color = scene.trace_ray(point);
            c.put_pixel(x, y, color);
        }
    }
}
