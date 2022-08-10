use crate::camera::Camera;
use crate::light::{Material, PointLight};
use crate::shapes::{Object, Plane, Sphere};
use crate::transform::Tr;
use crate::world::World;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;

/// Contains all information about the world except the objects.
#[derive(Deserialize, Debug, PartialEq)]
struct Data {
    camera: Camera,
    light: PointLight,
    // TODO: these two should be optional fields
    materials: Materials,
    transforms: Transforms,
    objects: Vec<ObjectRepr>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct CameraRepr {
    pub width: usize,
    pub height: usize,
    pub field_of_view: f64,
    pub from: (f64, f64, f64),
    pub to: (f64, f64, f64),
    pub up: (f64, f64, f64),
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct PointLightRepr {
    pub at: (f64, f64, f64),
    pub color: (f64, f64, f64),
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum MaterialRepr {
    // TODO these shouldn't be compulsory either. Because everything extends from the default
    // material.
    Complete {
        color: (f64, f64, f64),
        diffuse: f64,
        ambient: f64,
        specular: f64,
        shininess: f64,
        reflective: f64,
        refractive_index: f64,
        transparency: f64,
    },
    Extends {
        extends: String,
        color: Option<(f64, f64, f64)>,
        diffuse: Option<f64>,
        ambient: Option<f64>,
        specular: Option<f64>,
        shininess: Option<f64>,
        reflective: Option<f64>,
        refractive_index: Option<f64>,
        transparency: Option<f64>,
    },
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct MaterialReprs(pub HashMap<String, MaterialRepr>);

/// A map of material names to materials.
#[derive(Deserialize, Debug, PartialEq)]
#[serde(try_from = "MaterialReprs")]
pub struct Materials(pub HashMap<String, Material>);

/// Represents some problem with YAML parsing.
#[derive(Debug)]
pub enum ErrParseYaml {
    /// When some definition extends from another in a loop.
    RecursiveDefinition,
    KeyNotExists(String),
    Unsupported,
    UnknownTransformation(String),
    UnknownMaterial(String),
    SerdeError(serde_yaml::Error),
}

impl Display for ErrParseYaml {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ErrParseYaml::*;
        match self {
            RecursiveDefinition => write!(f, "Recursive definition in yaml"),
            KeyNotExists(key) => write!(f, "Key does not exist; key={key}"),
            Unsupported => write!(f, "An unsupported operation"),
            UnknownTransformation(tr) => write!(f, "Unknown transformation; tr={tr}"),
            UnknownMaterial(mat) => write!(f, "Unknown material; mat={mat}"),
            SerdeError(err) => write!(f, "Serde error; err={err}"),
        }
    }
}

/// Given a map of material definitions where some are incomplete and extend from others, attempt
/// to convert an incomplete one to a complete material definition.
pub fn complete_material(
    key: &str,
    map: &mut HashMap<String, MaterialRepr>,
    seen: &mut HashSet<String>,
) -> Result<(), ErrParseYaml> {
    // The seen set contains things which we have seen in previous recursive calls. If we encounter
    // the same key, then we have gone in a loop and cannot resolve this material.
    if seen.contains(key) {
        return Err(ErrParseYaml::RecursiveDefinition);
    }
    seen.insert(key.to_string()); // mark as seen

    // Remove the current material here, and then insert it later. Just to avoid cloning.
    let got = map
        .get(key)
        .cloned()
        .ok_or(ErrParseYaml::UnknownMaterial(key.to_string()))?;
    match got {
        MaterialRepr::Complete { .. } => Ok(()),
        MaterialRepr::Extends {
            extends,
            color,
            diffuse,
            ambient,
            specular,
            shininess,
            reflective,
            transparency,
            refractive_index,
        } => {
            // Recursively complete this material.
            complete_material(&extends, map, seen)?;
            let ext = map
                .get(&extends)
                .cloned()
                .ok_or(ErrParseYaml::KeyNotExists(extends))?;
            match ext {
                MaterialRepr::Extends { .. } => panic!("material should be complete"),
                MaterialRepr::Complete {
                    color: c,
                    diffuse: d,
                    ambient: a,
                    specular: sp,
                    shininess: sh,
                    reflective: r,
                    transparency: t,
                    refractive_index: ri,
                } => {
                    let res = MaterialRepr::Complete {
                        color: color.unwrap_or(c),
                        diffuse: diffuse.unwrap_or(d),
                        ambient: ambient.unwrap_or(a),
                        specular: specular.unwrap_or(sp),
                        shininess: shininess.unwrap_or(sh),
                        reflective: reflective.unwrap_or(r),
                        transparency: transparency.unwrap_or(t),
                        refractive_index: refractive_index.unwrap_or(ri),
                    };
                    map.insert(key.to_string(), res);
                    Ok(())
                }
            }
        }
    }
}

impl TryFrom<MaterialReprs> for Materials {
    type Error = ErrParseYaml;
    fn try_from(mut v: MaterialReprs) -> Result<Self, Self::Error> {
        let keys: Vec<String> = v.0.keys().cloned().collect();

        // Convert every material which extends to a complete kind.
        for key in keys {
            complete_material(&key, &mut v.0, &mut HashSet::new())?;
        }
        let mut res = HashMap::new();
        for (name, mat) in v.0 {
            res.insert(name, mat.try_into()?);
        }
        Ok(Materials(res))
    }
}

impl TryFrom<MaterialRepr> for Material {
    type Error = ErrParseYaml;
    fn try_from(value: MaterialRepr) -> Result<Self, Self::Error> {
        match value {
            MaterialRepr::Extends { .. } => Err(ErrParseYaml::Unsupported),
            MaterialRepr::Complete {
                color,
                diffuse,
                ambient,
                specular,
                shininess,
                reflective,
                transparency,
                refractive_index,
            } => Ok(Material::default()
                .with_color(color.into())
                .with_diffuse(diffuse)
                .with_ambient(ambient)
                .with_specular(specular)
                .with_reflective(reflective)
                .with_shininess(shininess)
                .with_transparency(transparency)
                .with_refractive_index(refractive_index)),
        }
    }
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum TransformRepr {
    /// References another transformation.
    Ref(String),
    /// Transformations involving one parameter, i.e. the three rotations.
    OneParam(String, f64),
    /// Transformations involving three parameters, i.e. translation and scaling.
    ThreeParam(String, f64, f64, f64),
    /// Transformations involving six parameters, i.e. shearing.
    SixParam(String, f64, f64, f64, f64, f64, f64),
}

#[derive(Deserialize, Debug)]
struct TransformReprs(pub HashMap<String, Vec<TransformRepr>>);

#[derive(Deserialize, Debug, PartialEq)]
#[serde(try_from = "TransformReprs")]
struct Transforms(pub HashMap<String, Tr>);

pub fn complete_transform(
    key: &str,
    map: &mut HashMap<String, Vec<TransformRepr>>,
    seen: &mut HashSet<String>,
) -> Result<(), ErrParseYaml> {
    // The seen set contains things which we have seen in previous recursive calls. If we encounter
    // the same key, then we have gone in a loop and cannot resolve this material.
    if seen.contains(key) {
        return Err(ErrParseYaml::RecursiveDefinition);
    }
    seen.insert(key.to_string()); // mark as seen

    let trs = map
        .remove(key)
        .ok_or(ErrParseYaml::KeyNotExists(key.to_string()))?;
    let mut new_trs: Vec<TransformRepr> = vec![];
    for tr in trs {
        match tr {
            TransformRepr::Ref(r) => {
                complete_transform(&r, map, seen)?;
                let mut xs = map.get(&r).cloned().expect("transform should exist");
                new_trs.append(&mut xs);
            }
            _ => new_trs.push(tr),
        }
    }
    map.insert(key.to_string(), new_trs);
    Ok(())
}

impl TryFrom<TransformReprs> for Transforms {
    type Error = ErrParseYaml;
    fn try_from(mut v: TransformReprs) -> Result<Self, Self::Error> {
        let keys: Vec<String> = v.0.keys().cloned().collect();

        // Convert every transform which extends to a complete kind.
        for key in keys {
            complete_transform(&key, &mut v.0, &mut HashSet::new())?;
        }
        let mut res = HashMap::new();
        for (name, trs) in v.0 {
            res.insert(
                name,
                trs.into_iter()
                    .map(|tr| Tr::try_from(tr).unwrap())
                    .fold(Tr::new(), |x, y| x.and(y)),
            );
        }
        Ok(Transforms(res))
    }
}

impl TryFrom<TransformRepr> for Tr {
    type Error = ErrParseYaml;
    fn try_from(value: TransformRepr) -> Result<Self, Self::Error> {
        use TransformRepr::*;
        match value {
            Ref(_) => Err(ErrParseYaml::Unsupported),
            OneParam(name, v) => match name.as_str() {
                "rotate_x" => Ok(Tr::new().rotate_x(v)),
                "rotate_y" => Ok(Tr::new().rotate_y(v)),
                "rotate_z" => Ok(Tr::new().rotate_z(v)),
                _ => Err(ErrParseYaml::UnknownTransformation(name)),
            },
            ThreeParam(name, x, y, z) => match name.as_str() {
                "translate" => Ok(Tr::new().translate(x, y, z)),
                "scale" => Ok(Tr::new().scale(x, y, z)),
                _ => Err(ErrParseYaml::UnknownTransformation(name)),
            },
            SixParam(name, x1, x2, x3, y1, y2, y3) => match name.as_str() {
                "shear" => Ok(Tr::new().shear(x1, x2, x3, y1, y2, y3)),
                _ => Err(ErrParseYaml::UnknownTransformation(name)),
            },
        }
    }
}

/// The kinds of objects which can be expressed in YAML.
#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Shape {
    Sphere,
    Plane,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
enum MaterialDefn {
    /// References a material.
    Ref(String),
    /// Does not reference any other material. Just extends from the default.
    Defined {
        color: Option<(f64, f64, f64)>,
        diffuse: Option<f64>,
        ambient: Option<f64>,
        specular: Option<f64>,
        reflective: Option<f64>,
    },
}

#[derive(Deserialize, Debug, PartialEq)]
struct ObjectRepr {
    #[serde(rename = "type")]
    typ: Shape,
    material: MaterialDefn,
    transform: Vec<TransformRepr>,
}

/// Generates a list of objects from their representations. Since some of the representations might
/// contain references, we need to pass in maps for materials and transformations.
fn generate_objects(
    xs: &[ObjectRepr],
    mats: &HashMap<String, Material>,
    trs: &HashMap<String, Tr>,
) -> Result<Vec<Object>, ErrParseYaml> {
    let mut res: Vec<Object> = vec![];
    for x in xs {
        // For each object, we need to get the material, transformation, and the shape.
        let mat = match &x.material {
            MaterialDefn::Ref(name) => mats
                .get(name)
                .cloned()
                .ok_or(ErrParseYaml::UnknownMaterial(name.to_string()))?,
            MaterialDefn::Defined {
                color,
                diffuse,
                ambient,
                specular,
                reflective,
            } => {
                // TODO: find a better way to do this. Very awkward now.
                let mat = Material::default();
                Material::default()
                    .with_color(color.map(|c| c.into()).unwrap_or(mat.color().into()))
                    .with_diffuse(diffuse.unwrap_or(mat.diffuse()))
                    .with_ambient(ambient.unwrap_or(mat.ambient()))
                    .with_specular(specular.unwrap_or(mat.specular()))
                    .with_reflective(reflective.unwrap_or(mat.reflective()))
            }
        };

        let mut transform = Tr::new();
        for tr in &x.transform {
            let x = match tr {
                TransformRepr::Ref(name) => trs
                    .get(name)
                    .cloned()
                    .ok_or(ErrParseYaml::UnknownTransformation(name.to_string()))?,
                _ => Tr::try_from(tr.clone())?,
            };
            transform = transform.and(x);
        }

        let shape = match x.typ {
            Shape::Plane => Plane::default()
                .with_material(mat)
                .with_transform(transform)
                .as_object(),
            Shape::Sphere => Sphere::default()
                .with_material(mat)
                .with_transform(transform)
                .as_object(),
        };

        res.push(shape);
    }

    Ok(res)
}

/// Given a YAML string, generates a PPM string representing the rendered world.
pub fn gen_world(yml: &str) -> Result<String, ErrParseYaml> {
    let data = serde_yaml::from_str::<Data>(yml).map_err(|e| ErrParseYaml::SerdeError(e))?;
    let objects = generate_objects(&data.objects, &data.materials.0, &data.transforms.0)?;

    let world = World::new().with_light(data.light).with_objects(objects);
    let output = data.camera.render(&world);
    Ok(output.to_ppm())
}

#[cfg(test)]
mod tests {
    use super::{
        generate_objects, Data, MaterialDefn, ObjectRepr, Shape, TransformRepr, Transforms,
    };
    use crate::camera::Camera;
    use crate::color::Color;
    use crate::light::{Material, PointLight};
    use crate::shapes::{Plane, Sphere};
    use crate::transform::{view_transform, Tr};
    use crate::yaml::Materials;
    use crate::{p, v};
    use std::collections::HashMap;

    /// A complete definition of a world in YAML.
    const TEST_YAML: &str = include_str!("./spec.yml");

    // Here are the expected results from some of the sections of the YAML file.
    lazy_static! {
        static ref CAMERA: Camera = Camera::new(100, 100, 0.785).with_transform(view_transform(
            p!(-6, 6, -10),
            p!(6, 0, 6),
            v!(-0.45, 1.0, 0.0),
        ));
        static ref LIGHT: PointLight = PointLight::new(p!(50, 100, -50), Color::new(1.0, 1.0, 1.0));
        static ref MATERIALS: HashMap<String, Material> = {
            let mut mats = HashMap::new();
            let white = Material::default()
                .with_color(Color::new(1.0, 1.0, 1.0))
                .with_diffuse(0.7)
                .with_ambient(0.1)
                .with_specular(0.0)
                .with_reflective(0.1);
            mats.insert("white".to_string(), white.clone());
            mats.insert(
                "blue".to_string(),
                white.with_color(Color::new(0.537, 0.831, 0.914)),
            );
            mats
        };
        static ref TRANSFORMS: HashMap<String, Tr> = {
            let mut trs = HashMap::new();
            let standard = Tr::new().translate(1.0, -1.0, 1.0).scale(0.5, 0.5, 0.5);
            let large = standard.scale(3.5, 3.5, 3.5);
            trs.insert("standard".to_string(), standard);
            trs.insert("large".to_string(), large);
            trs
        };
    }

    #[test]
    fn deserialize_camera() {
        let yaml = r#"
width: 100
height: 100
field_of_view: 0.785
from: [ -6, 6, -10 ]
to: [ 6, 0, 6 ]
up: [ -0.45, 1, 0 ]"#;
        let got: Camera = serde_yaml::from_str(yaml).expect("deserializes camera");
        assert_eq!(got, *CAMERA);
    }

    #[test]
    fn deserialize_light() {
        let yaml = r#"
at: [ 50, 100, -50 ]
color: [ 1, 1, 1 ]"#;
        let got: PointLight = serde_yaml::from_str(yaml).expect("deserializes light");
        assert_eq!(got, *LIGHT);
    }

    #[test]
    fn deserialize_material_definitions() {
        let yaml = r#"
white:
    color: [ 1, 1, 1 ]
    diffuse: 0.7
    ambient: 0.1
    specular: 0.0
    reflective: 0.1
blue:
    extends: white
    color: [ 0.537, 0.831, 0.914 ]"#;
        let got: Materials =
            serde_yaml::from_str(yaml).expect("deserializes multiple material definitions");
        assert_eq!(got, Materials(MATERIALS.clone()));
    }

    #[test]
    fn fails_on_recursive_material_definition() {
        let yaml = r#"
white:
    extends: blue
    color: [ 1, 1, 1 ]
    diffuse: 0.7
    ambient: 0.1
    specular: 0.0
blue:
    extends: white
    color: [ 0.537, 0.831, 0.914 ]"#;
        let got = serde_yaml::from_str::<Materials>(yaml);
        assert!(got.is_err());
    }

    #[test]
    fn deserialize_transform_definitions() {
        let yaml = r#"
standard:
    - [ translate, 1, -1, 1 ]
    - [ scale, 0.5, 0.5, 0.5 ]
large:
    - standard
    - [ scale, 3.5, 3.5, 3.5 ]"#;
        let got: Transforms =
            serde_yaml::from_str(yaml).expect("deserializes multiple transform definitions");
        assert_eq!(got, Transforms(TRANSFORMS.clone()));
    }

    #[test]
    fn fails_on_recursive_transform_definition() {
        let yaml = r#"
standard:
    - large
    - [ translate, 1, -1, 1 ]
    - [ scale, 0.5, 0.5, 0.5 ]
large:
    - standard
    - [ scale, 3.5, 3.5, 3.5 ]"#;
        let got = serde_yaml::from_str::<Transforms>(yaml);
        assert!(got.is_err());
    }

    #[test]
    fn deserialize_objects_repr() {
        let yaml = r#"
- type: sphere
  material: white
  transform:
    - large
- type: plane
  material:
    color: [ 1, 1, 1 ]
    ambient: 1
    diffuse: 0
    specular: 0
  transform:
    - [ rotate_x, 1.5707963267948966 ] # pi/2
    - [ translate, 0, 0, 500 ]"#;
        let got: Vec<ObjectRepr> = serde_yaml::from_str(yaml).expect("deserializes objects");
        let want = vec![
            ObjectRepr {
                typ: Shape::Sphere,
                material: MaterialDefn::Ref("white".to_string()),
                transform: vec![TransformRepr::Ref("large".to_string())],
            },
            ObjectRepr {
                typ: Shape::Plane,
                material: MaterialDefn::Defined {
                    color: Some((1.0, 1.0, 1.0)),
                    ambient: Some(1.0),
                    diffuse: Some(0.0),
                    specular: Some(0.0),
                    reflective: None,
                },
                transform: vec![
                    TransformRepr::OneParam("rotate_x".to_string(), 1.5707963267948966),
                    TransformRepr::ThreeParam("translate".to_string(), 0.0, 0.0, 500.0),
                ],
            },
        ];
        assert_eq!(got, want);
    }

    #[test]
    fn deserialize_objects() {
        let yaml = r#"
- type: sphere
  material: white
  transform:
    - large
- type: plane
  material:
    color: [ 1, 1, 1 ]
    ambient: 1
    diffuse: 0
    specular: 0
  transform:
    - [ rotate_x, 1.5707963267948966 ] # pi/2
    - [ translate, 0, 0, 500 ]"#;
        let xs: Vec<ObjectRepr> = serde_yaml::from_str(yaml).expect("deserializes objects");
        let got =
            generate_objects(&xs, &*MATERIALS, &*TRANSFORMS).expect("should generate objects");

        let want = vec![
            Sphere::default()
                .with_material(MATERIALS.get("white").cloned().unwrap())
                .with_transform(TRANSFORMS.get("large").cloned().unwrap())
                .as_object(),
            Plane::default()
                .with_material(
                    Material::default()
                        .with_color(Color::white())
                        .with_ambient(1.0)
                        .with_diffuse(0.0)
                        .with_specular(0.0),
                )
                .with_transform(
                    Tr::new()
                        .rotate_x(1.5707963267948966)
                        .translate(0.0, 0.0, 500.0),
                )
                .as_object(),
        ];

        assert_eq!(got, want);
    }

    #[test]
    fn deserialize_data() {
        let got = serde_yaml::from_str::<Data>(TEST_YAML).expect("deserializes data");

        let want = Data {
            camera: *CAMERA,
            light: *LIGHT,
            materials: Materials(MATERIALS.clone()),
            transforms: Transforms(TRANSFORMS.clone()),
            // TODO: this is not legit. We need to check if the two vectors contain the same
            // elements, not that they are exactly equal.
            objects: vec![
                ObjectRepr {
                    typ: Shape::Sphere,
                    material: MaterialDefn::Ref("white".to_string()),
                    transform: vec![TransformRepr::Ref("large".to_string())],
                },
                ObjectRepr {
                    typ: Shape::Plane,
                    material: MaterialDefn::Defined {
                        color: Some((1.0, 1.0, 1.0)),
                        ambient: Some(1.0),
                        diffuse: Some(0.0),
                        specular: Some(0.0),
                        reflective: None,
                    },
                    transform: vec![
                        TransformRepr::OneParam("rotate_x".to_string(), 1.5707963267948966),
                        TransformRepr::ThreeParam("translate".to_string(), 0.0, 0.0, 500.0),
                    ],
                },
            ],
        };

        assert_eq!(got, want);
    }

    // TODO test what happens if some key is not defined!
}
