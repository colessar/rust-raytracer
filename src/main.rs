mod vec3;
mod image;
mod ray;
mod hittable;
mod camera;
mod material;

use image::{Image, Pixel};
use vec3::Vec3;
use ray::Ray;
use hittable::{HitRecord, Hittable, sphere::Sphere};
use std::{fs::File, io::Write, iter::repeat_with, rc::Rc};
use crate::{camera::Camera, material::{Lambertian, Dielectric}, material::Metal};


fn trace_ray(r: &Ray, world: &Vec<Box<dyn Hittable>>, max_depth: usize) -> Pixel {

    if max_depth <= 0 {
        return Pixel::black();
    }

    let mut t_closest_so_far = f64::INFINITY;
    let mut rec: Option<HitRecord> = None;

    for obj in world {
        let result = obj.hit(&r, 0.001, t_closest_so_far);

        if let Some(temp_rec) = result {
            t_closest_so_far = temp_rec.t_value();
            rec = Some(temp_rec);
        }
    }

    if let Some(final_rec) = rec {

        if let Some((attenuation, new_ray)) = final_rec.material().scatter(r, final_rec) {
            let pixel = trace_ray(&new_ray, world, max_depth - 1);
    
            return Pixel::new(
                (attenuation.x * pixel.r as f64) as u8,
                (attenuation.y * pixel.g as f64) as u8,
                (attenuation.z * pixel.b as f64) as u8
            );
        }

        return Pixel::black();
    }

    let w = 0.5*(r.direction().y + 1.0);
    let white: Vec3 = Vec3::new(1.0, 1.0, 1.0);
    let blue: Vec3 = Vec3::new(0.5, 0.7, 1.0);
    let color = white*(1.0 - w) + blue*w;
    Pixel::new(
        (color.x * 255.0) as u8,
        (color.y * 255.0) as u8,
        (color.z * 255.0) as u8
    )
}

fn main() -> std::io::Result<()> {
    const ASPECT_RATIO: f64 = 16.0/9.0;

    const VP_HEIGHT: f64 = 2.0;
    const VP_WIDTH: f64 = VP_HEIGHT * ASPECT_RATIO;

    const IMG_HEIGHT: usize = 400;
    const IMG_WIDTH: usize = (IMG_HEIGHT as f64 * ASPECT_RATIO) as usize;

    const FOCAL_LENGTH: f64 = 1.0;

    const SAMPLES_PER_PIXEL: usize = 100;

    const MAX_DEPTH: usize = 50;

    let origin = Vec3::new(0.0, 0.0, 0.0);
    let camera = Camera::new(&origin, VP_HEIGHT, VP_WIDTH, FOCAL_LENGTH);

    let mut image = Image::new(IMG_HEIGHT, IMG_WIDTH);

    let material_ground = Rc::new(Lambertian::new(&Vec3::new(0.8, 0.8, 0.0)));
    let material_left = Rc::new(Dielectric::new(1.5));
    let material_right = Rc::new(Metal::new(&Vec3::new(0.8, 0.6, 0.2), 0.0));
    let material_center = Rc::new(Lambertian::new(&Vec3::new(0.1, 0.2, 0.5)));

    let sphere_center = Sphere::new(
        &Vec3::new( 0.0, 0.0, -1.0),
        0.5,
        material_center
    );

    let sphere_left = Sphere::new(
        &Vec3::new(-1.0, 0.0, -1.0),
        0.5,
        material_left
    );

    let sphere_right = Sphere::new(
        &Vec3::new(1.0, 0.0, -1.0),
        0.5,
        material_right
    );

    let sphere_ground = Sphere::new(
        &Vec3::new(0.0, -100.5, -1.0),
        100.0,
        material_ground
    );

    let mut world: Vec<Box<dyn Hittable>> = Vec::new();
    world.push(Box::new(sphere_ground));
    world.push(Box::new(sphere_left));
    world.push(Box::new(sphere_right));
    world.push(Box::new(sphere_center));

    for y in 0..image.height() {
        for x in 0..image.width() {
            let avg_sample: Vec3 = repeat_with(|| fastrand::f64())
                                    .take(SAMPLES_PER_PIXEL)
                                    .map(|random_val| ((x as f64 + random_val) / (image.width() as f64 - 1.0), (y as f64 + random_val) / (image.height() as f64 - 1.0)))
                                    .map(|(u,v)| camera.get_ray(u, v))
                                    .map(|ray| trace_ray(&ray, &world, MAX_DEPTH))
                                    .map(|pixel| Vec3::new(pixel.r as f64, pixel.g as f64, pixel.b as f64))
                                    .fold(Vec3::new(0.0, 0.0, 0.0), |acc, v| acc + v) / SAMPLES_PER_PIXEL as f64;
            let avg_sample = (avg_sample / 255.0).sqrt() * 256.0;
            let pixel = Pixel::from_vec3(&avg_sample);
            image.insert_pixel_at(x, image.height() - y - 1, &pixel);
        }
    }

    let mut f = File::create("test.ppm")?;
    f.write_all(image.to_ppm().as_bytes())?;
    Ok(())
}