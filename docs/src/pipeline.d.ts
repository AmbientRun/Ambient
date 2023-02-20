// pipeline.json
export type u32 = number;
export type f32 = number;
export type Vec2 = [number, number];
export type Vec3 = [number, number, number];
export type Vec4 = [number, number, number, number];
export type EntityData = {[component_id: string]: any};
export type AssetUrl = string;

export type Pipeline = {
  /// Undocumented!
  pipeline: {
    type: "Models",
    /// Undocumented!
    importer?: {
      type: "Regular",
    } | {
      type: "UnityModels",
      /// Undocumented!
      use_prefabs: boolean,
    } | {
      type: "Quixel",
    },
    /// Use assimp as the importer; this will support more file formats, but is less well integrated
    force_assimp?: boolean,
    /// Undocumented!
    collider?: {
      type: "None",
    } | {
      type: "FromModel",
      /// Undocumented!
      flip_normals?: boolean,
      /// Undocumented!
      reverse_indices?: boolean,
    } | {
      type: "Character",
      /// Undocumented!
      radius?: f32,
      /// Undocumented!
      height?: f32,
    },
    /// Undocumented!
    collider_type?: "Static" | "Dynamic" | "TriggerArea" | "Picking",
    /// Undocumented!
    cap_texture_sizes?: "X128" | "X256" | "X512" | "X1024" | "X2048" | "X4096" | {"Custom": u32},
    /// Treats all assets in the pipeline as variations, and outputs a single asset which is a collection of all assets
    collection_of_variants?: boolean,
    /// Output objects which can be spawned from server-side scripts
    output_objects?: boolean,
    /// Undocumented!
    output_animations?: boolean,
    /// Add components to server side objects
    object_components?: EntityData,
    /// Undocumented!
    material_overrides?: {
      /// Undocumented!
      filter: {
        type: "All",
      } | {
        type: "ByName",
        /// Undocumented!
        name: string,
      },
      /// Undocumented!
      material: {
        /// Undocumented!
        name?: string,
        /// Undocumented!
        source?: string,
        /// Undocumented!
        base_color?: AssetUrl,
        /// Undocumented!
        opacity?: AssetUrl,
        /// Undocumented!
        normalmap?: AssetUrl,
        /// Undocumented!
        metallic_roughness?: AssetUrl,
        /// Undocumented!
        base_color_factor?: Vec4,
        /// Undocumented!
        emissive_factor?: Vec4,
        /// Undocumented!
        transparent?: boolean,
        /// Undocumented!
        alpha_cutoff?: f32,
        /// Undocumented!
        double_sided?: boolean,
        /// Undocumented!
        metallic?: f32,
        /// Undocumented!
        roughness?: f32,
        /// Undocumented!
        specular?: AssetUrl,
        /// Undocumented!
        specular_exponent?: f32,
      },
    }[],
    /// Undocumented!
    transforms?: ({
      type: "RotateYUpToZUp",
    } | {
      type: "RotateX",
      /// Undocumented!
      deg: f32,
    } | {
      type: "RotateY",
      /// Undocumented!
      deg: f32,
    } | {
      type: "RotateZ",
      /// Undocumented!
      deg: f32,
    } | {
      type: "Scale",
      /// Undocumented!
      scale: f32,
    } | {
      type: "Translate",
      /// Undocumented!
      translation: Vec3,
    } | {
      type: "ScaleAABB",
      /// Undocumented!
      scale: f32,
    } | {
      type: "ScaleAnimations",
      /// Undocumented!
      scale: f32,
    } | {
      type: "SetRoot",
      /// Undocumented!
      name: string,
    } | {
      type: "Center",
    })[],
  } | {
    type: "Materials",
    /// Undocumented!
    importer: {
      type: "Single",
      /// Undocumented!
      name?: string,
      /// Undocumented!
      source?: string,
      /// Undocumented!
      base_color?: AssetUrl,
      /// Undocumented!
      opacity?: AssetUrl,
      /// Undocumented!
      normalmap?: AssetUrl,
      /// Undocumented!
      metallic_roughness?: AssetUrl,
      /// Undocumented!
      base_color_factor?: Vec4,
      /// Undocumented!
      emissive_factor?: Vec4,
      /// Undocumented!
      transparent?: boolean,
      /// Undocumented!
      alpha_cutoff?: f32,
      /// Undocumented!
      double_sided?: boolean,
      /// Undocumented!
      metallic?: f32,
      /// Undocumented!
      roughness?: f32,
      /// Undocumented!
      specular?: AssetUrl,
      /// Undocumented!
      specular_exponent?: f32,
    } | {
      type: "Quixel",
    },
    /// Undocumented!
    output_decals?: boolean,
  } | {
    type: "Audio",
  },
  /// Filter sources; this is a list of glob patterns for accepted files
  /// All files are accepted if this is empty
  sources?: string[],
  /// Tags to apply to the output resources
  tags?: string[],
  /// Categories ot apply to the output resources
  categories?: string[][],
}