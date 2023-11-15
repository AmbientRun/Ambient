#[allow(
    unused,
    clippy::unit_arg,
    clippy::let_and_return,
    clippy::approx_constant,
    clippy::unused_unit
)]
mod raw {
    pub mod mwrcsok65na7owrbdococ5sthr3ccskc {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("mwrcsok65na7owrbdococ5sthr3ccskc")
                    .expect("Failed to get package entity - was it despawned?")
            });
            *ENTITY
        }
        pub mod player {
            pub mod class {
                #[doc = r" Auto-generated component definitions."]
                pub mod components {
                    use ambient_api::{
                        ecs::{Component, __internal_get_component},
                        once_cell::sync::Lazy,
                        prelude::*,
                    };
                    static IS_CLASS: Lazy<Component<()>> = Lazy::new(|| {
                        __internal_get_component(
                            "mwrcsok65na7owrbdococ5sthr3ccskc::player::class::is_class",
                        )
                    });
                    #[doc = "**Is Class**: Is a player class\n\n*Attributes*: Networked, Debuggable"]
                    pub fn is_class() -> Component<()> {
                        *IS_CLASS
                    }
                    static NAME: Lazy<Component<String>> = Lazy::new(|| {
                        __internal_get_component(
                            "mwrcsok65na7owrbdococ5sthr3ccskc::player::class::name",
                        )
                    });
                    #[doc = "**Name**: Name of the player class\n\n*Attributes*: Networked, Debuggable"]
                    pub fn name() -> Component<String> {
                        *NAME
                    }
                    static DESCRIPTION: Lazy<Component<String>> = Lazy::new(|| {
                        __internal_get_component(
                            "mwrcsok65na7owrbdococ5sthr3ccskc::player::class::description",
                        )
                    });
                    #[doc = "**Description**: Description of the player class\n\n*Attributes*: Networked, Debuggable"]
                    pub fn description() -> Component<String> {
                        *DESCRIPTION
                    }
                    static ICON_URL: Lazy<Component<String>> = Lazy::new(|| {
                        __internal_get_component(
                            "mwrcsok65na7owrbdococ5sthr3ccskc::player::class::icon_url",
                        )
                    });
                    #[doc = "**Icon URL**: URL of the icon for the player class\n\n*Attributes*: Networked, Debuggable"]
                    pub fn icon_url() -> Component<String> {
                        *ICON_URL
                    }
                    static DEF_REF: Lazy<Component<EntityId>> = Lazy::new(|| {
                        __internal_get_component(
                            "mwrcsok65na7owrbdococ5sthr3ccskc::player::class::def_ref",
                        )
                    });
                    #[doc = "**Class Definition Ref**: When attached to a class, indicates that it should draw its properties from this entity.\n\n*Attributes*: Networked, Debuggable"]
                    pub fn def_ref() -> Component<EntityId> {
                        *DEF_REF
                    }
                }
            }
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use ambient_api::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static VEHICLE_REF: Lazy<Component<EntityId>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::player::vehicle_ref",
                    )
                });
                #[doc = "**Player's Vehicle**: The player's vehicle\n\n*Attributes*: Networked, Debuggable"]
                pub fn vehicle_ref() -> Component<EntityId> {
                    *VEHICLE_REF
                }
                static CHARACTER_REF: Lazy<Component<EntityId>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::player::character_ref",
                    )
                });
                #[doc = "**Player's Character**: The player's character\n\n*Attributes*: Networked, Debuggable"]
                pub fn character_ref() -> Component<EntityId> {
                    *CHARACTER_REF
                }
                static CLASS_REF: Lazy<Component<EntityId>> = Lazy::new(|| {
                    __internal_get_component("mwrcsok65na7owrbdococ5sthr3ccskc::player::class_ref")
                });
                #[doc = "**Class**: The player's class\n\n*Attributes*: Networked, Debuggable"]
                pub fn class_ref() -> Component<EntityId> {
                    *CLASS_REF
                }
                static INPUT_DIRECTION: Lazy<Component<Vec2>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::player::input_direction",
                    )
                });
                #[doc = "**Input Direction**: Input direction\n\n*Attributes*: Debuggable"]
                pub fn input_direction() -> Component<Vec2> {
                    *INPUT_DIRECTION
                }
                static INPUT_JUMP: Lazy<Component<bool>> = Lazy::new(|| {
                    __internal_get_component("mwrcsok65na7owrbdococ5sthr3ccskc::player::input_jump")
                });
                #[doc = "**Jump**: Jump\n\n*Attributes*: Debuggable"]
                pub fn input_jump() -> Component<bool> {
                    *INPUT_JUMP
                }
                static INPUT_FIRE: Lazy<Component<bool>> = Lazy::new(|| {
                    __internal_get_component("mwrcsok65na7owrbdococ5sthr3ccskc::player::input_fire")
                });
                #[doc = "**Fire**: Fire\n\n*Attributes*: Debuggable"]
                pub fn input_fire() -> Component<bool> {
                    *INPUT_FIRE
                }
                static INPUT_USE: Lazy<Component<bool>> = Lazy::new(|| {
                    __internal_get_component("mwrcsok65na7owrbdococ5sthr3ccskc::player::input_use")
                });
                #[doc = "**Use**: Use\n\n*Attributes*: Debuggable"]
                pub fn input_use() -> Component<bool> {
                    *INPUT_USE
                }
                static INPUT_SPRINT: Lazy<Component<bool>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::player::input_sprint",
                    )
                });
                #[doc = "**Sprint**: Sprint\n\n*Attributes*: Debuggable"]
                pub fn input_sprint() -> Component<bool> {
                    *INPUT_SPRINT
                }
                static INPUT_RESPAWN: Lazy<Component<bool>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::player::input_respawn",
                    )
                });
                #[doc = "**Respawn**: Respawn\n\n*Attributes*: Debuggable"]
                pub fn input_respawn() -> Component<bool> {
                    *INPUT_RESPAWN
                }
                static INPUT_AIM_DIRECTION: Lazy<Component<Vec2>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::player::input_aim_direction",
                    )
                });
                #[doc = "**Aim Direction**: Aim direction in degrees from the centre\n\n*Attributes*: Debuggable"]
                pub fn input_aim_direction() -> Component<Vec2> {
                    *INPUT_AIM_DIRECTION
                }
                static INPUT_AIM_RAY_ORIGIN: Lazy<Component<Vec3>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::player::input_aim_ray_origin",
                    )
                });
                #[doc = "**Aim Ray Origin**: Origin of the aim ray\n\n*Attributes*: Debuggable"]
                pub fn input_aim_ray_origin() -> Component<Vec3> {
                    *INPUT_AIM_RAY_ORIGIN
                }
                static INPUT_AIM_RAY_DIRECTION: Lazy<Component<Vec3>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::player::input_aim_ray_direction",
                    )
                });
                #[doc = "**Aim Ray Direction**: Direction of the aim ray\n\n*Attributes*: Debuggable"]
                pub fn input_aim_ray_direction() -> Component<Vec3> {
                    *INPUT_AIM_RAY_DIRECTION
                }
            }
        }
        pub mod character {
            pub mod head {
                #[doc = r" Auto-generated component definitions."]
                pub mod components {
                    use ambient_api::{
                        ecs::{Component, __internal_get_component},
                        once_cell::sync::Lazy,
                        prelude::*,
                    };
                    static CAMERA_REF: Lazy<Component<EntityId>> = Lazy::new(|| {
                        __internal_get_component(
                            "mwrcsok65na7owrbdococ5sthr3ccskc::character::head::camera_ref",
                        )
                    });
                    #[doc = "**Head Camera**: The camera attached to the character's head\n\n*Attributes*: Networked, Debuggable"]
                    pub fn camera_ref() -> Component<EntityId> {
                        *CAMERA_REF
                    }
                }
            }
            pub mod def {
                #[doc = r" Auto-generated component definitions."]
                pub mod components {
                    use ambient_api::{
                        ecs::{Component, __internal_get_component},
                        once_cell::sync::Lazy,
                        prelude::*,
                    };
                    static MODEL_URL: Lazy<Component<String>> = Lazy::new(|| {
                        __internal_get_component(
                            "mwrcsok65na7owrbdococ5sthr3ccskc::character::def::model_url",
                        )
                    });
                    #[doc = "**Model URL**: URL of the model for the character def\n\n*Attributes*: Networked, Debuggable"]
                    pub fn model_url() -> Component<String> {
                        *MODEL_URL
                    }
                }
            }
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use ambient_api::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static IS_CHARACTER: Lazy<Component<()>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::character::is_character",
                    )
                });
                #[doc = "**Is Character**: Is a player character\n\n*Attributes*: Networked, Debuggable"]
                pub fn is_character() -> Component<()> {
                    *IS_CHARACTER
                }
                static PLAYER_REF: Lazy<Component<EntityId>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::character::player_ref",
                    )
                });
                #[doc = "**Character's Player**: The player controlling the character\n\n*Attributes*: Networked, Debuggable"]
                pub fn player_ref() -> Component<EntityId> {
                    *PLAYER_REF
                }
                static LAST_USE_TIME: Lazy<Component<Duration>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::character::last_use_time",
                    )
                });
                #[doc = "**Last Use Time**: The last time the player tried to use something\n\n*Attributes*: Networked, Debuggable"]
                pub fn last_use_time() -> Component<Duration> {
                    *LAST_USE_TIME
                }
                static DEF_REF: Lazy<Component<EntityId>> = Lazy::new(|| {
                    __internal_get_component("mwrcsok65na7owrbdococ5sthr3ccskc::character::def_ref")
                });
                #[doc = "**Character Definition Ref**: When attached to a character, indicates that it should draw its properties from this entity.\n\n*Attributes*: Networked, Debuggable"]
                pub fn def_ref() -> Component<EntityId> {
                    *DEF_REF
                }
            }
        }
        pub mod vehicle {
            pub mod def {
                pub mod thruster {
                    #[doc = r" Auto-generated component definitions."]
                    pub mod components {
                        use ambient_api::{
                            ecs::{Component, __internal_get_component},
                            once_cell::sync::Lazy,
                            prelude::*,
                        };
                        static OFFSETS: Lazy<Component<Vec<Vec2>>> = Lazy::new(|| {
                            __internal_get_component(
                                "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::thruster::offsets",
                            )
                        });
                        #[doc = "**Thruster Offsets**: Offsets of the thrusters from the center of the vehicle\n\n*Attributes*: Networked, Debuggable"]
                        pub fn offsets() -> Component<Vec<Vec2>> {
                            *OFFSETS
                        }
                        static K_P: Lazy<Component<f32>> = Lazy::new(|| {
                            __internal_get_component(
                                "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::thruster::k_p",
                            )
                        });
                        #[doc = "**Thruster K_p**: Proportional gain for the thrusters\n\n*Attributes*: Networked, Debuggable"]
                        pub fn k_p() -> Component<f32> {
                            *K_P
                        }
                        static K_D: Lazy<Component<f32>> = Lazy::new(|| {
                            __internal_get_component(
                                "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::thruster::k_d",
                            )
                        });
                        #[doc = "**Thruster K_d**: Derivative gain for the thrusters\n\n*Attributes*: Networked, Debuggable"]
                        pub fn k_d() -> Component<f32> {
                            *K_D
                        }
                        static TARGET: Lazy<Component<f32>> = Lazy::new(|| {
                            __internal_get_component(
                                "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::thruster::target",
                            )
                        });
                        #[doc = "**Thruster Target Altitude**: Target altitude for the thrusters\n\n*Attributes*: Networked, Debuggable"]
                        pub fn target() -> Component<f32> {
                            *TARGET
                        }
                        static MAX_STRENGTH: Lazy<Component<f32>> = Lazy::new(|| {
                            __internal_get_component ("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::thruster::max_strength")
                        });
                        #[doc = "**Thruster Max Strength**: Maximum strength of the thrusters\n\n*Attributes*: Networked, Debuggable"]
                        pub fn max_strength() -> Component<f32> {
                            *MAX_STRENGTH
                        }
                    }
                }
                pub mod input {
                    #[doc = r" Auto-generated component definitions."]
                    pub mod components {
                        use ambient_api::{
                            ecs::{Component, __internal_get_component},
                            once_cell::sync::Lazy,
                            prelude::*,
                        };
                        static FORWARD_FORCE: Lazy<Component<f32>> = Lazy::new(|| {
                            __internal_get_component ("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::forward_force")
                        });
                        #[doc = "**Forward Force**: Forward force applied to the vehicle on input\n\n*Attributes*: Networked, Debuggable"]
                        pub fn forward_force() -> Component<f32> {
                            *FORWARD_FORCE
                        }
                        static BACKWARD_FORCE: Lazy<Component<f32>> = Lazy::new(|| {
                            __internal_get_component ("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::backward_force")
                        });
                        #[doc = "**Backward Force**: Backward force applied to the vehicle on input\n\n*Attributes*: Networked, Debuggable"]
                        pub fn backward_force() -> Component<f32> {
                            *BACKWARD_FORCE
                        }
                        static FORWARD_OFFSET: Lazy<Component<Vec2>> = Lazy::new(|| {
                            __internal_get_component ("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::forward_offset")
                        });
                        #[doc = "**Forward Offset**: Offset of the forward force from the center of the vehicle. Typically at the back of the vehicle.\n\n*Attributes*: Networked, Debuggable"]
                        pub fn forward_offset() -> Component<Vec2> {
                            *FORWARD_OFFSET
                        }
                        static SIDE_FORCE: Lazy<Component<f32>> = Lazy::new(|| {
                            __internal_get_component(
                                "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::side_force",
                            )
                        });
                        #[doc = "**Side Force**: Side force applied to the vehicle on input\n\n*Attributes*: Networked, Debuggable"]
                        pub fn side_force() -> Component<f32> {
                            *SIDE_FORCE
                        }
                        static SIDE_OFFSET: Lazy<Component<Vec2>> = Lazy::new(|| {
                            __internal_get_component ("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::side_offset")
                        });
                        #[doc = "**Side Offset**: Offset of the side force from the center of the vehicle. Typically at the front of the vehicle.\n\n*Attributes*: Networked, Debuggable"]
                        pub fn side_offset() -> Component<Vec2> {
                            *SIDE_OFFSET
                        }
                        static JUMP_FORCE: Lazy<Component<f32>> = Lazy::new(|| {
                            __internal_get_component(
                                "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::jump_force",
                            )
                        });
                        #[doc = "**Jump Force**: Jump force applied to the vehicle on input\n\n*Attributes*: Networked, Debuggable"]
                        pub fn jump_force() -> Component<f32> {
                            *JUMP_FORCE
                        }
                        static PITCH_STRENGTH: Lazy<Component<f32>> = Lazy::new(|| {
                            __internal_get_component ("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::pitch_strength")
                        });
                        #[doc = "**Pitch Strength**: Strength of the pitch applied to the applicable thrusters of the vehicle on input\n\n*Attributes*: Networked, Debuggable"]
                        pub fn pitch_strength() -> Component<f32> {
                            *PITCH_STRENGTH
                        }
                        static TURNING_STRENGTH: Lazy<Component<f32>> = Lazy::new(|| {
                            __internal_get_component ("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::turning_strength")
                        });
                        #[doc = "**Turning Strength**: Strength of the turning applied to the applicable thrusters of the vehicle on input\n\n*Attributes*: Networked, Debuggable"]
                        pub fn turning_strength() -> Component<f32> {
                            *TURNING_STRENGTH
                        }
                        static JUMP_TIMEOUT: Lazy<Component<Duration>> = Lazy::new(|| {
                            __internal_get_component ("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::jump_timeout")
                        });
                        #[doc = "**Jump Timeout**: Timeout between jumps\n\n*Attributes*: Networked, Debuggable"]
                        pub fn jump_timeout() -> Component<Duration> {
                            *JUMP_TIMEOUT
                        }
                        static AIM_DIRECTION_LIMITS: Lazy<Component<Vec2>> = Lazy::new(|| {
                            __internal_get_component ("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::aim_direction_limits")
                        });
                        #[doc = "**Aim Direction Limits**: Limits of the aim direction in degrees from the centre\n\n*Attributes*: Networked, Debuggable"]
                        pub fn aim_direction_limits() -> Component<Vec2> {
                            *AIM_DIRECTION_LIMITS
                        }
                    }
                }
                pub mod slowdown {
                    #[doc = r" Auto-generated component definitions."]
                    pub mod components {
                        use ambient_api::{
                            ecs::{Component, __internal_get_component},
                            once_cell::sync::Lazy,
                            prelude::*,
                        };
                        static LINEAR_STRENGTH: Lazy<Component<f32>> = Lazy::new(|| {
                            __internal_get_component ("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::slowdown::linear_strength")
                        });
                        #[doc = "**Linear Strength**: Strength of the linear slowdown applied to the vehicle\n\n*Attributes*: Networked, Debuggable"]
                        pub fn linear_strength() -> Component<f32> {
                            *LINEAR_STRENGTH
                        }
                        static ANGULAR_STRENGTH: Lazy<Component<f32>> = Lazy::new(|| {
                            __internal_get_component ("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::slowdown::angular_strength")
                        });
                        #[doc = "**Angular Strength**: Strength of the angular slowdown applied to the vehicle\n\n*Attributes*: Networked, Debuggable"]
                        pub fn angular_strength() -> Component<f32> {
                            *ANGULAR_STRENGTH
                        }
                        static ANGULAR_DELAY: Lazy<Component<Duration>> = Lazy::new(|| {
                            __internal_get_component ("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::slowdown::angular_delay")
                        });
                        #[doc = "**Angular Delay**: Time to wait before applying angular slowdown\n\n*Attributes*: Networked, Debuggable"]
                        pub fn angular_delay() -> Component<Duration> {
                            *ANGULAR_DELAY
                        }
                    }
                }
                #[doc = r" Auto-generated component definitions."]
                pub mod components {
                    use ambient_api::{
                        ecs::{Component, __internal_get_component},
                        once_cell::sync::Lazy,
                        prelude::*,
                    };
                    static IS_DEF: Lazy<Component<()>> = Lazy::new(|| {
                        __internal_get_component(
                            "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::is_def",
                        )
                    });
                    #[doc = "**Is Def**: Is a vehicle def\n\n*Attributes*: Networked, Debuggable"]
                    pub fn is_def() -> Component<()> {
                        *IS_DEF
                    }
                    static NAME: Lazy<Component<String>> = Lazy::new(|| {
                        __internal_get_component(
                            "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::name",
                        )
                    });
                    #[doc = "**Name**: Name of the vehicle def\n\n*Attributes*: Networked, Debuggable"]
                    pub fn name() -> Component<String> {
                        *NAME
                    }
                    static MODEL_URL: Lazy<Component<String>> = Lazy::new(|| {
                        __internal_get_component(
                            "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::model_url",
                        )
                    });
                    #[doc = "**Model URL**: URL of the model for the vehicle def\n\n*Attributes*: Networked, Debuggable"]
                    pub fn model_url() -> Component<String> {
                        *MODEL_URL
                    }
                    static MODEL_SCALE: Lazy<Component<f32>> = Lazy::new(|| {
                        __internal_get_component(
                            "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::model_scale",
                        )
                    });
                    #[doc = "**Model Scale**: Scale of the model for the vehicle def\n\n*Attributes*: Networked, Debuggable"]
                    pub fn model_scale() -> Component<f32> {
                        *MODEL_SCALE
                    }
                }
            }
            pub mod client {
                #[doc = r" Auto-generated component definitions."]
                pub mod components {
                    use ambient_api::{
                        ecs::{Component, __internal_get_component},
                        once_cell::sync::Lazy,
                        prelude::*,
                    };
                    static SPEED_KPH: Lazy<Component<f32>> = Lazy::new(|| {
                        __internal_get_component(
                            "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::client::speed_kph",
                        )
                    });
                    #[doc = "**Speed (KPH)**: Speed of the vehicle in kilometers per hour."]
                    pub fn speed_kph() -> Component<f32> {
                        *SPEED_KPH
                    }
                }
            }
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use ambient_api::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static IS_VEHICLE: Lazy<Component<()>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::is_vehicle",
                    )
                });
                #[doc = "**Is Vehicle**: Is a vehicle\n\n*Attributes*: Networked, Debuggable"]
                pub fn is_vehicle() -> Component<()> {
                    *IS_VEHICLE
                }
                static DRIVER_REF: Lazy<Component<EntityId>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::driver_ref",
                    )
                });
                #[doc = "**Vehicle Driver**: A vehicle's driver, if present\n\n*Attributes*: Networked, Debuggable"]
                pub fn driver_ref() -> Component<EntityId> {
                    *DRIVER_REF
                }
                static DEF_REF: Lazy<Component<EntityId>> = Lazy::new(|| {
                    __internal_get_component("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def_ref")
                });
                #[doc = "**Vehicle Definition Ref**: When attached to a vehicle entity, indicates that it should draw its properties from this entity.\n\n*Attributes*: Networked, Debuggable"]
                pub fn def_ref() -> Component<EntityId> {
                    *DEF_REF
                }
                static AIMABLE_WEAPON_REFS: Lazy<Component<Vec<EntityId>>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::aimable_weapon_refs",
                    )
                });
                #[doc = "**Aimable Weapon Refs**: All aimable weapons for this vehicle. If present in this array, their rotations will be automatically updated to match the aim position. Assumes that the weapons are a child of the vehicle.\n\n*Attributes*: Networked, Debuggable"]
                pub fn aimable_weapon_refs() -> Component<Vec<EntityId>> {
                    *AIMABLE_WEAPON_REFS
                }
                static LAST_DISTANCES: Lazy<Component<Vec<f32>>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::last_distances",
                    )
                });
                #[doc = "**Last Distances**: Distances from the ground for each vehicle probe\n\n*Attributes*: Networked, Debuggable"]
                pub fn last_distances() -> Component<Vec<f32>> {
                    *LAST_DISTANCES
                }
                static LAST_JUMP_TIME: Lazy<Component<Duration>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::last_jump_time",
                    )
                });
                #[doc = "**Last Jump Time**: The last time the player jumped\n\n*Attributes*: Networked, Debuggable"]
                pub fn last_jump_time() -> Component<Duration> {
                    *LAST_JUMP_TIME
                }
                static LAST_SLOWDOWN_TIME: Lazy<Component<Duration>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::last_slowdown_time",
                    )
                });
                #[doc = "**Last Slowdown Time**: The last time the player's vehicle was slowed down\n\n*Attributes*: Networked, Debuggable"]
                pub fn last_slowdown_time() -> Component<Duration> {
                    *LAST_SLOWDOWN_TIME
                }
                static LAST_UPSIDE_DOWN_TIME: Lazy<Component<Duration>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::last_upside_down_time",
                    )
                });
                #[doc = "**Last Upside Down Time**: The last time the player's vehicle was known to be upside down. Added when first detected, removed when no longer the case. Used to respawn the vehicle after some time.\n\n*Attributes*: Networked, Debuggable"]
                pub fn last_upside_down_time() -> Component<Duration> {
                    *LAST_UPSIDE_DOWN_TIME
                }
                static INPUT_DIRECTION: Lazy<Component<Vec2>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::input_direction",
                    )
                });
                #[doc = "**Input Direction**: Input direction\n\n*Attributes*: Debuggable"]
                pub fn input_direction() -> Component<Vec2> {
                    *INPUT_DIRECTION
                }
                static INPUT_JUMP: Lazy<Component<bool>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::input_jump",
                    )
                });
                #[doc = "**Jump**: Jump\n\n*Attributes*: Debuggable"]
                pub fn input_jump() -> Component<bool> {
                    *INPUT_JUMP
                }
                static INPUT_FIRE: Lazy<Component<bool>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::input_fire",
                    )
                });
                #[doc = "**Fire**: Fire\n\n*Attributes*: Debuggable"]
                pub fn input_fire() -> Component<bool> {
                    *INPUT_FIRE
                }
                static AIM_POSITION: Lazy<Component<Vec3>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::aim_position",
                    )
                });
                #[doc = "**Aim Position**: Position of the aim target\n\n*Attributes*: Debuggable"]
                pub fn aim_position() -> Component<Vec3> {
                    *AIM_POSITION
                }
            }
        }
        pub mod spawnpoint {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use ambient_api::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static IS_SPAWNPOINT: Lazy<Component<()>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::spawnpoint::is_spawnpoint",
                    )
                });
                #[doc = "**Is Spawnpoint**: Is a spawnpoint\n\n*Attributes*: Networked, Debuggable"]
                pub fn is_spawnpoint() -> Component<()> {
                    *IS_SPAWNPOINT
                }
                static RADIUS: Lazy<Component<f32>> = Lazy::new(|| {
                    __internal_get_component("mwrcsok65na7owrbdococ5sthr3ccskc::spawnpoint::radius")
                });
                #[doc = "**Radius**: Radius of the spawnpoint\n\n*Attributes*: Networked, Debuggable"]
                pub fn radius() -> Component<f32> {
                    *RADIUS
                }
            }
        }
        pub mod weapon {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use ambient_api::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static FIRE: Lazy<Component<bool>> = Lazy::new(|| {
                    __internal_get_component("mwrcsok65na7owrbdococ5sthr3ccskc::weapon::fire")
                });
                #[doc = "**Fire**: When on, the weapon will attempt to fire.\n\n*Attributes*: Networked, Debuggable"]
                pub fn fire() -> Component<bool> {
                    *FIRE
                }
            }
        }
        #[doc = r" Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept."]
        #[doc = r""]
        #[doc = r" They do not have any runtime representation outside of the components that compose them."]
        pub mod concepts {
            use ambient_api::{
                global::serde::{self, Deserialize, Serialize},
                prelude::*,
            };
            #[doc = "**Player Class**: A player class\n\n**Required**:\n- `is_class`: Is a player class\n- `name`: Name of the player class\n- `description`: Description of the player class\n- `icon_url`: URL of the icon for the player class\n- `def_ref`: When attached to a class, indicates that it should draw its properties from this entity."]
            #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct PlayerClass {
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::player::class::is_class`\n\n**Component description**: Is a player class\n\n"]
                pub is_class: (),
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::player::class::name`\n\n**Component description**: Name of the player class\n\n"]
                pub name: String,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::player::class::description`\n\n**Component description**: Description of the player class\n\n"]
                pub description: String,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::player::class::icon_url`\n\n**Component description**: URL of the icon for the player class\n\n"]
                pub icon_url: String,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::player::class::def_ref`\n\n**Component description**: When attached to a class, indicates that it should draw its properties from this entity.\n\n"]
                pub def_ref: EntityId,
            }
            impl Concept for PlayerClass {
                fn make(self) -> Entity {
                    let mut entity = Entity :: new () . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: is_class () , self . is_class) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: name () , self . name) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: description () , self . description) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: icon_url () , self . icon_url) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: def_ref () , self . def_ref) ;
                    entity
                }
                fn get_spawned(id: EntityId) -> Option<Self> {
                    Some (Self { is_class : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: is_class ()) ? , name : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: name ()) ? , description : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: description ()) ? , icon_url : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: icon_url ()) ? , def_ref : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: def_ref ()) ? , })
                }
                fn get_unspawned(entity: &Entity) -> Option<Self> {
                    Some (Self { is_class : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: is_class ()) ? , name : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: name ()) ? , description : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: description ()) ? , icon_url : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: icon_url ()) ? , def_ref : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: def_ref ()) ? , })
                }
                fn contained_by_spawned(id: EntityId) -> bool {
                    entity :: has_components (id , & [& crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: is_class () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: name () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: description () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: icon_url () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: def_ref ()])
                }
                fn contained_by_unspawned(entity: &Entity) -> bool {
                    entity . has_components (& [& crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: is_class () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: name () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: description () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: icon_url () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: def_ref ()])
                }
            }
            impl ConceptComponents for PlayerClass {
                type Required = (
                    Component<()>,
                    Component<String>,
                    Component<String>,
                    Component<String>,
                    Component<EntityId>,
                );
                type Optional = ();
                fn required() -> Self::Required {
                    (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: is_class () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: name () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: description () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: icon_url () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: def_ref () ,)
                }
                fn optional() -> Self::Optional {
                    ()
                }
                fn from_required_data(required: <Self::Required as ComponentsTuple>::Data) -> Self {
                    Self {
                        is_class: required.0,
                        name: required.1,
                        description: required.2,
                        icon_url: required.3,
                        def_ref: required.4,
                    }
                }
            }
            #[doc = "**Character**: A character with a physical representation\n\n**Required**:\n- `translation`: The translation/position of this entity.\n- `rotation`: The rotation of this entity.\n- `health`: This game object's health. \"Standard\" health is 100 HP.\n- `max_health`: Maximum health of the object. 100 HP is \"standard.\"\n- `is_character`: Is a player character\n- `player_ref`: The player controlling the character\n- `def_ref`: When attached to a character, indicates that it should draw its properties from this entity.\n\n\n**Optional**:\n- `last_use_time`: The last time the player tried to use something"]
            #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct Character {
                #[doc = "**Component**: `ambient_core::transform::translation`\n\n**Component description**: The translation/position of this entity.\n\n"]
                pub translation: Vec3,
                #[doc = "**Component**: `ambient_core::transform::rotation`\n\n**Component description**: The rotation of this entity.\n\n"]
                pub rotation: Quat,
                #[doc = "**Component**: `hvxms7i2px7krvkm23sxfjxsjqlcmtb5::health`\n\n**Component description**: This game object's health. \"Standard\" health is 100 HP.\n\n"]
                pub health: f32,
                #[doc = "**Component**: `hvxms7i2px7krvkm23sxfjxsjqlcmtb5::max_health`\n\n**Component description**: Maximum health of the object. 100 HP is \"standard.\"\n\n"]
                pub max_health: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::character::is_character`\n\n**Component description**: Is a player character\n\n"]
                pub is_character: (),
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::character::player_ref`\n\n**Component description**: The player controlling the character\n\n"]
                pub player_ref: EntityId,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::character::def_ref`\n\n**Component description**: When attached to a character, indicates that it should draw its properties from this entity.\n\n"]
                pub def_ref: EntityId,
                #[doc = r" Optional components."]
                pub optional: CharacterOptional,
            }
            #[doc = "Optional part of [Character]."]
            #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct CharacterOptional {
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::character::last_use_time`\n\n**Component description**: The last time the player tried to use something\n\n"]
                pub last_use_time: Option<Duration>,
            }
            impl Concept for Character {
                fn make(self) -> Entity {
                    let mut entity = Entity :: new () . with (ambient_api :: core :: transform :: components :: translation () , self . translation) . with (ambient_api :: core :: transform :: components :: rotation () , self . rotation) . with (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: health () , self . health) . with (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , self . max_health) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: is_character () , self . is_character) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: player_ref () , self . player_ref) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: def_ref () , self . def_ref) ;
                    if let Some(last_use_time) = self.optional.last_use_time {
                        entity . set (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: last_use_time () , last_use_time) ;
                    }
                    entity
                }
                fn get_spawned(id: EntityId) -> Option<Self> {
                    Some (Self { translation : entity :: get_component (id , ambient_api :: core :: transform :: components :: translation ()) ? , rotation : entity :: get_component (id , ambient_api :: core :: transform :: components :: rotation ()) ? , health : entity :: get_component (id , crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: health ()) ? , max_health : entity :: get_component (id , crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health ()) ? , is_character : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: is_character ()) ? , player_ref : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: player_ref ()) ? , def_ref : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: def_ref ()) ? , optional : CharacterOptional { last_use_time : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: last_use_time ()) , } })
                }
                fn get_unspawned(entity: &Entity) -> Option<Self> {
                    Some (Self { translation : entity . get (ambient_api :: core :: transform :: components :: translation ()) ? , rotation : entity . get (ambient_api :: core :: transform :: components :: rotation ()) ? , health : entity . get (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: health ()) ? , max_health : entity . get (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health ()) ? , is_character : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: is_character ()) ? , player_ref : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: player_ref ()) ? , def_ref : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: def_ref ()) ? , optional : CharacterOptional { last_use_time : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: last_use_time ()) , } })
                }
                fn contained_by_spawned(id: EntityId) -> bool {
                    entity :: has_components (id , & [& ambient_api :: core :: transform :: components :: translation () , & ambient_api :: core :: transform :: components :: rotation () , & crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: health () , & crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: is_character () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: player_ref () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: def_ref ()])
                }
                fn contained_by_unspawned(entity: &Entity) -> bool {
                    entity . has_components (& [& ambient_api :: core :: transform :: components :: translation () , & ambient_api :: core :: transform :: components :: rotation () , & crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: health () , & crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: is_character () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: player_ref () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: def_ref ()])
                }
            }
            impl ConceptComponents for Character {
                type Required = (
                    Component<Vec3>,
                    Component<Quat>,
                    Component<f32>,
                    Component<f32>,
                    Component<()>,
                    Component<EntityId>,
                    Component<EntityId>,
                );
                type Optional = (Component<Duration>,);
                fn required() -> Self::Required {
                    (ambient_api :: core :: transform :: components :: translation () , ambient_api :: core :: transform :: components :: rotation () , crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: health () , crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: is_character () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: player_ref () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: def_ref () ,)
                }
                fn optional() -> Self::Optional {
                    (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: last_use_time () ,)
                }
                fn from_required_data(required: <Self::Required as ComponentsTuple>::Data) -> Self {
                    Self {
                        translation: required.0,
                        rotation: required.1,
                        health: required.2,
                        max_health: required.3,
                        is_character: required.4,
                        player_ref: required.5,
                        def_ref: required.6,
                        optional: Default::default(),
                    }
                }
            }
            #[doc = "**Character Definition**: Definition for a character\n\n**Required**:\n- `max_health`: Maximum health of the object. 100 HP is \"standard.\"\n- `model_url`: URL of the model for the character def\n- `speed`: The speed the unit can walk at\n- `run_speed_multiplier`: The speed the unit can run at\n- `strafe_speed_multiplier`: The speed the unit can strafe at"]
            #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct CharacterDef {
                #[doc = "**Component**: `hvxms7i2px7krvkm23sxfjxsjqlcmtb5::max_health`\n\n**Component description**: Maximum health of the object. 100 HP is \"standard.\"\n\n"]
                pub max_health: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::character::def::model_url`\n\n**Component description**: URL of the model for the character def\n\n"]
                pub model_url: String,
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::speed`\n\n**Component description**: The speed the unit can walk at\n\n"]
                pub speed: f32,
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::run_speed_multiplier`\n\n**Component description**: The speed the unit can run at\n\n"]
                pub run_speed_multiplier: f32,
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::strafe_speed_multiplier`\n\n**Component description**: The speed the unit can strafe at\n\n"]
                pub strafe_speed_multiplier: f32,
            }
            impl Concept for CharacterDef {
                fn make(self) -> Entity {
                    let mut entity = Entity :: new () . with (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , self . max_health) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: def :: components :: model_url () , self . model_url) . with (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: speed () , self . speed) . with (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_speed_multiplier () , self . run_speed_multiplier) . with (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: strafe_speed_multiplier () , self . strafe_speed_multiplier) ;
                    entity
                }
                fn get_spawned(id: EntityId) -> Option<Self> {
                    Some (Self { max_health : entity :: get_component (id , crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health ()) ? , model_url : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: def :: components :: model_url ()) ? , speed : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: speed ()) ? , run_speed_multiplier : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_speed_multiplier ()) ? , strafe_speed_multiplier : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: strafe_speed_multiplier ()) ? , })
                }
                fn get_unspawned(entity: &Entity) -> Option<Self> {
                    Some (Self { max_health : entity . get (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health ()) ? , model_url : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: def :: components :: model_url ()) ? , speed : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: speed ()) ? , run_speed_multiplier : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_speed_multiplier ()) ? , strafe_speed_multiplier : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: strafe_speed_multiplier ()) ? , })
                }
                fn contained_by_spawned(id: EntityId) -> bool {
                    entity :: has_components (id , & [& crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: def :: components :: model_url () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: speed () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_speed_multiplier () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: strafe_speed_multiplier ()])
                }
                fn contained_by_unspawned(entity: &Entity) -> bool {
                    entity . has_components (& [& crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: def :: components :: model_url () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: speed () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_speed_multiplier () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: strafe_speed_multiplier ()])
                }
            }
            impl ConceptComponents for CharacterDef {
                type Required = (
                    Component<f32>,
                    Component<String>,
                    Component<f32>,
                    Component<f32>,
                    Component<f32>,
                );
                type Optional = ();
                fn required() -> Self::Required {
                    (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: def :: components :: model_url () , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: speed () , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_speed_multiplier () , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: strafe_speed_multiplier () ,)
                }
                fn optional() -> Self::Optional {
                    ()
                }
                fn from_required_data(required: <Self::Required as ComponentsTuple>::Data) -> Self {
                    Self {
                        max_health: required.0,
                        model_url: required.1,
                        speed: required.2,
                        run_speed_multiplier: required.3,
                        strafe_speed_multiplier: required.4,
                    }
                }
            }
            #[doc = "**Vehicle Definition**: Definition for a vehicle\n\n**Required**:\n- `density`: The density of this entity.\nThis is used to update the `mass` when the entity is rescaled.\n- `cube_collider`: If attached, this entity will have a cube physics collider.\n`x, y, z` is the size of the cube.\n- `max_health`: Maximum health of the object. 100 HP is \"standard.\"\n- `offsets`: Offsets of the thrusters from the center of the vehicle\n- `k_p`: Proportional gain for the thrusters\n- `k_d`: Derivative gain for the thrusters\n- `target`: Target altitude for the thrusters\n- `max_strength`: Maximum strength of the thrusters\n- `forward_force`: Forward force applied to the vehicle on input\n- `backward_force`: Backward force applied to the vehicle on input\n- `forward_offset`: Offset of the forward force from the center of the vehicle. Typically at the back of the vehicle.\n- `side_force`: Side force applied to the vehicle on input\n- `side_offset`: Offset of the side force from the center of the vehicle. Typically at the front of the vehicle.\n- `jump_force`: Jump force applied to the vehicle on input\n- `aim_direction_limits`: Limits of the aim direction in degrees from the centre\n- `pitch_strength`: Strength of the pitch applied to the applicable thrusters of the vehicle on input\n- `turning_strength`: Strength of the turning applied to the applicable thrusters of the vehicle on input\n- `jump_timeout`: Timeout between jumps\n- `linear_strength`: Strength of the linear slowdown applied to the vehicle\n- `angular_strength`: Strength of the angular slowdown applied to the vehicle\n- `angular_delay`: Time to wait before applying angular slowdown\n- `is_def`: Is a vehicle def\n- `name`: Name of the vehicle def\n- `model_url`: URL of the model for the vehicle def\n- `model_scale`: Scale of the model for the vehicle def"]
            #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct VehicleDef {
                #[doc = "**Component**: `ambient_core::physics::density`\n\n**Component description**: The density of this entity.\nThis is used to update the `mass` when the entity is rescaled.\n\n"]
                pub density: f32,
                #[doc = "**Component**: `ambient_core::physics::cube_collider`\n\n**Component description**: If attached, this entity will have a cube physics collider.\n`x, y, z` is the size of the cube.\n\n"]
                pub cube_collider: Vec3,
                #[doc = "**Component**: `hvxms7i2px7krvkm23sxfjxsjqlcmtb5::max_health`\n\n**Component description**: Maximum health of the object. 100 HP is \"standard.\"\n\n"]
                pub max_health: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::thruster::offsets`\n\n**Component description**: Offsets of the thrusters from the center of the vehicle\n\n"]
                pub offsets: Vec<Vec2>,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::thruster::k_p`\n\n**Component description**: Proportional gain for the thrusters\n\n"]
                pub k_p: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::thruster::k_d`\n\n**Component description**: Derivative gain for the thrusters\n\n"]
                pub k_d: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::thruster::target`\n\n**Component description**: Target altitude for the thrusters\n\n"]
                pub target: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::thruster::max_strength`\n\n**Component description**: Maximum strength of the thrusters\n\n"]
                pub max_strength: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::forward_force`\n\n**Component description**: Forward force applied to the vehicle on input\n\n"]
                pub forward_force: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::backward_force`\n\n**Component description**: Backward force applied to the vehicle on input\n\n"]
                pub backward_force: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::forward_offset`\n\n**Component description**: Offset of the forward force from the center of the vehicle. Typically at the back of the vehicle.\n\n"]
                pub forward_offset: Vec2,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::side_force`\n\n**Component description**: Side force applied to the vehicle on input\n\n"]
                pub side_force: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::side_offset`\n\n**Component description**: Offset of the side force from the center of the vehicle. Typically at the front of the vehicle.\n\n"]
                pub side_offset: Vec2,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::jump_force`\n\n**Component description**: Jump force applied to the vehicle on input\n\n"]
                pub jump_force: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::aim_direction_limits`\n\n**Component description**: Limits of the aim direction in degrees from the centre\n\n"]
                pub aim_direction_limits: Vec2,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::pitch_strength`\n\n**Component description**: Strength of the pitch applied to the applicable thrusters of the vehicle on input\n\n"]
                pub pitch_strength: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::turning_strength`\n\n**Component description**: Strength of the turning applied to the applicable thrusters of the vehicle on input\n\n"]
                pub turning_strength: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::jump_timeout`\n\n**Component description**: Timeout between jumps\n\n"]
                pub jump_timeout: Duration,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::slowdown::linear_strength`\n\n**Component description**: Strength of the linear slowdown applied to the vehicle\n\n"]
                pub linear_strength: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::slowdown::angular_strength`\n\n**Component description**: Strength of the angular slowdown applied to the vehicle\n\n"]
                pub angular_strength: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::slowdown::angular_delay`\n\n**Component description**: Time to wait before applying angular slowdown\n\n"]
                pub angular_delay: Duration,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::is_def`\n\n**Component description**: Is a vehicle def\n\n"]
                pub is_def: (),
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::name`\n\n**Component description**: Name of the vehicle def\n\n"]
                pub name: String,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::model_url`\n\n**Component description**: URL of the model for the vehicle def\n\n"]
                pub model_url: String,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::model_scale`\n\n**Component description**: Scale of the model for the vehicle def\n\n"]
                pub model_scale: f32,
            }
            impl Concept for VehicleDef {
                fn make(self) -> Entity {
                    let mut entity = Entity :: new () . with (ambient_api :: core :: physics :: components :: density () , self . density) . with (ambient_api :: core :: physics :: components :: cube_collider () , self . cube_collider) . with (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , self . max_health) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: offsets () , self . offsets) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: k_p () , self . k_p) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: k_d () , self . k_d) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: target () , self . target) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: max_strength () , self . max_strength) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: forward_force () , self . forward_force) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: backward_force () , self . backward_force) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: forward_offset () , self . forward_offset) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: side_force () , self . side_force) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: side_offset () , self . side_offset) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: jump_force () , self . jump_force) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: aim_direction_limits () , self . aim_direction_limits) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: pitch_strength () , self . pitch_strength) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: turning_strength () , self . turning_strength) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: jump_timeout () , self . jump_timeout) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: linear_strength () , self . linear_strength) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: angular_strength () , self . angular_strength) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: angular_delay () , self . angular_delay) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: is_def () , self . is_def) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: name () , self . name) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: model_url () , self . model_url) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: model_scale () , self . model_scale) ;
                    entity
                }
                fn get_spawned(id: EntityId) -> Option<Self> {
                    Some (Self { density : entity :: get_component (id , ambient_api :: core :: physics :: components :: density ()) ? , cube_collider : entity :: get_component (id , ambient_api :: core :: physics :: components :: cube_collider ()) ? , max_health : entity :: get_component (id , crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health ()) ? , offsets : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: offsets ()) ? , k_p : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: k_p ()) ? , k_d : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: k_d ()) ? , target : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: target ()) ? , max_strength : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: max_strength ()) ? , forward_force : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: forward_force ()) ? , backward_force : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: backward_force ()) ? , forward_offset : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: forward_offset ()) ? , side_force : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: side_force ()) ? , side_offset : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: side_offset ()) ? , jump_force : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: jump_force ()) ? , aim_direction_limits : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: aim_direction_limits ()) ? , pitch_strength : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: pitch_strength ()) ? , turning_strength : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: turning_strength ()) ? , jump_timeout : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: jump_timeout ()) ? , linear_strength : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: linear_strength ()) ? , angular_strength : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: angular_strength ()) ? , angular_delay : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: angular_delay ()) ? , is_def : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: is_def ()) ? , name : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: name ()) ? , model_url : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: model_url ()) ? , model_scale : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: model_scale ()) ? , })
                }
                fn get_unspawned(entity: &Entity) -> Option<Self> {
                    Some (Self { density : entity . get (ambient_api :: core :: physics :: components :: density ()) ? , cube_collider : entity . get (ambient_api :: core :: physics :: components :: cube_collider ()) ? , max_health : entity . get (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health ()) ? , offsets : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: offsets ()) ? , k_p : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: k_p ()) ? , k_d : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: k_d ()) ? , target : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: target ()) ? , max_strength : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: max_strength ()) ? , forward_force : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: forward_force ()) ? , backward_force : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: backward_force ()) ? , forward_offset : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: forward_offset ()) ? , side_force : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: side_force ()) ? , side_offset : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: side_offset ()) ? , jump_force : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: jump_force ()) ? , aim_direction_limits : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: aim_direction_limits ()) ? , pitch_strength : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: pitch_strength ()) ? , turning_strength : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: turning_strength ()) ? , jump_timeout : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: jump_timeout ()) ? , linear_strength : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: linear_strength ()) ? , angular_strength : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: angular_strength ()) ? , angular_delay : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: angular_delay ()) ? , is_def : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: is_def ()) ? , name : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: name ()) ? , model_url : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: model_url ()) ? , model_scale : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: model_scale ()) ? , })
                }
                fn contained_by_spawned(id: EntityId) -> bool {
                    entity :: has_components (id , & [& ambient_api :: core :: physics :: components :: density () , & ambient_api :: core :: physics :: components :: cube_collider () , & crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: offsets () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: k_p () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: k_d () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: target () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: max_strength () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: forward_force () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: backward_force () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: forward_offset () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: side_force () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: side_offset () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: jump_force () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: aim_direction_limits () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: pitch_strength () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: turning_strength () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: jump_timeout () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: linear_strength () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: angular_strength () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: angular_delay () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: is_def () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: name () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: model_url () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: model_scale ()])
                }
                fn contained_by_unspawned(entity: &Entity) -> bool {
                    entity . has_components (& [& ambient_api :: core :: physics :: components :: density () , & ambient_api :: core :: physics :: components :: cube_collider () , & crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: offsets () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: k_p () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: k_d () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: target () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: max_strength () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: forward_force () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: backward_force () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: forward_offset () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: side_force () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: side_offset () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: jump_force () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: aim_direction_limits () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: pitch_strength () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: turning_strength () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: jump_timeout () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: linear_strength () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: angular_strength () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: angular_delay () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: is_def () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: name () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: model_url () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: model_scale ()])
                }
            }
            impl ConceptComponents for VehicleDef {
                type Required = (
                    Component<f32>,
                    Component<Vec3>,
                    Component<f32>,
                    Component<Vec<Vec2>>,
                    Component<f32>,
                    Component<f32>,
                    Component<f32>,
                    Component<f32>,
                    Component<f32>,
                    Component<f32>,
                    Component<Vec2>,
                    Component<f32>,
                    Component<Vec2>,
                    Component<f32>,
                    Component<Vec2>,
                    Component<f32>,
                    Component<f32>,
                    Component<Duration>,
                    Component<f32>,
                    Component<f32>,
                    Component<Duration>,
                    Component<()>,
                    Component<String>,
                    Component<String>,
                    Component<f32>,
                );
                type Optional = ();
                fn required() -> Self::Required {
                    (ambient_api :: core :: physics :: components :: density () , ambient_api :: core :: physics :: components :: cube_collider () , crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: offsets () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: k_p () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: k_d () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: target () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: max_strength () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: forward_force () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: backward_force () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: forward_offset () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: side_force () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: side_offset () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: jump_force () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: aim_direction_limits () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: pitch_strength () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: turning_strength () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: jump_timeout () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: linear_strength () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: angular_strength () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: angular_delay () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: is_def () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: name () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: model_url () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: model_scale () ,)
                }
                fn optional() -> Self::Optional {
                    ()
                }
                fn from_required_data(required: <Self::Required as ComponentsTuple>::Data) -> Self {
                    Self {
                        density: required.0,
                        cube_collider: required.1,
                        max_health: required.2,
                        offsets: required.3,
                        k_p: required.4,
                        k_d: required.5,
                        target: required.6,
                        max_strength: required.7,
                        forward_force: required.8,
                        backward_force: required.9,
                        forward_offset: required.10,
                        side_force: required.11,
                        side_offset: required.12,
                        jump_force: required.13,
                        aim_direction_limits: required.14,
                        pitch_strength: required.15,
                        turning_strength: required.16,
                        jump_timeout: required.17,
                        linear_strength: required.18,
                        angular_strength: required.19,
                        angular_delay: required.20,
                        is_def: required.21,
                        name: required.22,
                        model_url: required.23,
                        model_scale: required.24,
                    }
                }
            }
            #[doc = "**Vehicle**: A vehicle with a physical representation\n\n**Required**:\n- `linear_velocity`: Linear velocity (meters/second) of this entity in the physics scene.\nUpdating this component will update the entity's linear velocity in the physics scene.\n\nNote that changing this component will forcibly set the velocity; changing the velocity every frame may lead to unexpected behavior, like gravity not working or collisions failing.\n\nIf you need to adjust the velocity each frame, consider applying a force using `physics` functions instead.\n- `angular_velocity`: Angular velocity (radians/second) of this entity in the physics scene.\nUpdating this component will update the entity's angular velocity in the physics scene.\n\nNote that changing this component will forcibly set the velocity; changing the velocity every frame may lead to unexpected behavior, like improper physics or collisions failing.\n\nIf you need to adjust the velocity each frame, consider applying an impulse using `physics` functions instead.\n- `physics_controlled`: If attached, this entity will be controlled by physics.\nNote that this requires the entity to have a collider.\n- `dynamic`: If this is true, the entity will be dynamic (i.e. be able to move). Otherwise, it will be static.\n- `density`: The density of this entity.\nThis is used to update the `mass` when the entity is rescaled.\n- `cube_collider`: If attached, this entity will have a cube physics collider.\n`x, y, z` is the size of the cube.\n- `local_to_world`: Transformation from the entity's local space to worldspace.\n- `translation`: The translation/position of this entity.\n- `rotation`: The rotation of this entity.\n- `health`: This game object's health. \"Standard\" health is 100 HP.\n- `max_health`: Maximum health of the object. 100 HP is \"standard.\"\n- `is_vehicle`: Is a vehicle\n- `last_distances`: Distances from the ground for each vehicle probe\n- `last_jump_time`: The last time the player jumped\n- `last_slowdown_time`: The last time the player's vehicle was slowed down\n- `def_ref`: When attached to a vehicle entity, indicates that it should draw its properties from this entity.\n- `input_direction`: Input direction\n- `input_jump`: Jump\n- `input_fire`: Fire\n\n\n**Optional**:\n- `driver_ref`: A vehicle's driver, if present\n- `last_upside_down_time`: The last time the player's vehicle was known to be upside down. Added when first detected, removed when no longer the case. Used to respawn the vehicle after some time.\n- `aim_position`: Position of the aim target\n- `aimable_weapon_refs`: All aimable weapons for this vehicle. If present in this array, their rotations will be automatically updated to match the aim position. Assumes that the weapons are a child of the vehicle."]
            #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct Vehicle {
                #[doc = "**Component**: `ambient_core::physics::linear_velocity`\n\n**Component description**: Linear velocity (meters/second) of this entity in the physics scene.\nUpdating this component will update the entity's linear velocity in the physics scene.\n\nNote that changing this component will forcibly set the velocity; changing the velocity every frame may lead to unexpected behavior, like gravity not working or collisions failing.\n\nIf you need to adjust the velocity each frame, consider applying a force using `physics` functions instead.\n\n"]
                pub linear_velocity: Vec3,
                #[doc = "**Component**: `ambient_core::physics::angular_velocity`\n\n**Component description**: Angular velocity (radians/second) of this entity in the physics scene.\nUpdating this component will update the entity's angular velocity in the physics scene.\n\nNote that changing this component will forcibly set the velocity; changing the velocity every frame may lead to unexpected behavior, like improper physics or collisions failing.\n\nIf you need to adjust the velocity each frame, consider applying an impulse using `physics` functions instead.\n\n"]
                pub angular_velocity: Vec3,
                #[doc = "**Component**: `ambient_core::physics::physics_controlled`\n\n**Component description**: If attached, this entity will be controlled by physics.\nNote that this requires the entity to have a collider.\n\n"]
                pub physics_controlled: (),
                #[doc = "**Component**: `ambient_core::physics::dynamic`\n\n**Component description**: If this is true, the entity will be dynamic (i.e. be able to move). Otherwise, it will be static.\n\n"]
                pub dynamic: bool,
                #[doc = "**Component**: `ambient_core::physics::density`\n\n**Component description**: The density of this entity.\nThis is used to update the `mass` when the entity is rescaled.\n\n"]
                pub density: f32,
                #[doc = "**Component**: `ambient_core::physics::cube_collider`\n\n**Component description**: If attached, this entity will have a cube physics collider.\n`x, y, z` is the size of the cube.\n\n"]
                pub cube_collider: Vec3,
                #[doc = "**Component**: `ambient_core::transform::local_to_world`\n\n**Component description**: Transformation from the entity's local space to worldspace.\n\n"]
                pub local_to_world: Mat4,
                #[doc = "**Component**: `ambient_core::transform::translation`\n\n**Component description**: The translation/position of this entity.\n\n"]
                pub translation: Vec3,
                #[doc = "**Component**: `ambient_core::transform::rotation`\n\n**Component description**: The rotation of this entity.\n\n"]
                pub rotation: Quat,
                #[doc = "**Component**: `hvxms7i2px7krvkm23sxfjxsjqlcmtb5::health`\n\n**Component description**: This game object's health. \"Standard\" health is 100 HP.\n\n"]
                pub health: f32,
                #[doc = "**Component**: `hvxms7i2px7krvkm23sxfjxsjqlcmtb5::max_health`\n\n**Component description**: Maximum health of the object. 100 HP is \"standard.\"\n\n"]
                pub max_health: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::is_vehicle`\n\n**Component description**: Is a vehicle\n\n"]
                pub is_vehicle: (),
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::last_distances`\n\n**Component description**: Distances from the ground for each vehicle probe\n\n"]
                pub last_distances: Vec<f32>,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::last_jump_time`\n\n**Component description**: The last time the player jumped\n\n"]
                pub last_jump_time: Duration,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::last_slowdown_time`\n\n**Component description**: The last time the player's vehicle was slowed down\n\n"]
                pub last_slowdown_time: Duration,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def_ref`\n\n**Component description**: When attached to a vehicle entity, indicates that it should draw its properties from this entity.\n\n"]
                pub def_ref: EntityId,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::input_direction`\n\n**Component description**: Input direction\n\n"]
                pub input_direction: Vec2,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::input_jump`\n\n**Component description**: Jump\n\n"]
                pub input_jump: bool,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::input_fire`\n\n**Component description**: Fire\n\n"]
                pub input_fire: bool,
                #[doc = r" Optional components."]
                pub optional: VehicleOptional,
            }
            #[doc = "Optional part of [Vehicle]."]
            #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct VehicleOptional {
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::driver_ref`\n\n**Component description**: A vehicle's driver, if present\n\n"]
                pub driver_ref: Option<EntityId>,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::last_upside_down_time`\n\n**Component description**: The last time the player's vehicle was known to be upside down. Added when first detected, removed when no longer the case. Used to respawn the vehicle after some time.\n\n"]
                pub last_upside_down_time: Option<Duration>,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::aim_position`\n\n**Component description**: Position of the aim target\n\n"]
                pub aim_position: Option<Vec3>,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::aimable_weapon_refs`\n\n**Component description**: All aimable weapons for this vehicle. If present in this array, their rotations will be automatically updated to match the aim position. Assumes that the weapons are a child of the vehicle.\n\n"]
                pub aimable_weapon_refs: Option<Vec<EntityId>>,
            }
            impl Concept for Vehicle {
                fn make(self) -> Entity {
                    let mut entity = Entity :: new () . with (ambient_api :: core :: physics :: components :: linear_velocity () , self . linear_velocity) . with (ambient_api :: core :: physics :: components :: angular_velocity () , self . angular_velocity) . with (ambient_api :: core :: physics :: components :: physics_controlled () , self . physics_controlled) . with (ambient_api :: core :: physics :: components :: dynamic () , self . dynamic) . with (ambient_api :: core :: physics :: components :: density () , self . density) . with (ambient_api :: core :: physics :: components :: cube_collider () , self . cube_collider) . with (ambient_api :: core :: transform :: components :: local_to_world () , self . local_to_world) . with (ambient_api :: core :: transform :: components :: translation () , self . translation) . with (ambient_api :: core :: transform :: components :: rotation () , self . rotation) . with (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: health () , self . health) . with (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , self . max_health) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: is_vehicle () , self . is_vehicle) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_distances () , self . last_distances) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_jump_time () , self . last_jump_time) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_slowdown_time () , self . last_slowdown_time) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: def_ref () , self . def_ref) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_direction () , self . input_direction) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_jump () , self . input_jump) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_fire () , self . input_fire) ;
                    if let Some(driver_ref) = self.optional.driver_ref {
                        entity . set (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: driver_ref () , driver_ref) ;
                    }
                    if let Some(last_upside_down_time) = self.optional.last_upside_down_time {
                        entity . set (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_upside_down_time () , last_upside_down_time) ;
                    }
                    if let Some(aim_position) = self.optional.aim_position {
                        entity . set (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: aim_position () , aim_position) ;
                    }
                    if let Some(aimable_weapon_refs) = self.optional.aimable_weapon_refs {
                        entity . set (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: aimable_weapon_refs () , aimable_weapon_refs) ;
                    }
                    entity
                }
                fn get_spawned(id: EntityId) -> Option<Self> {
                    Some (Self { linear_velocity : entity :: get_component (id , ambient_api :: core :: physics :: components :: linear_velocity ()) ? , angular_velocity : entity :: get_component (id , ambient_api :: core :: physics :: components :: angular_velocity ()) ? , physics_controlled : entity :: get_component (id , ambient_api :: core :: physics :: components :: physics_controlled ()) ? , dynamic : entity :: get_component (id , ambient_api :: core :: physics :: components :: dynamic ()) ? , density : entity :: get_component (id , ambient_api :: core :: physics :: components :: density ()) ? , cube_collider : entity :: get_component (id , ambient_api :: core :: physics :: components :: cube_collider ()) ? , local_to_world : entity :: get_component (id , ambient_api :: core :: transform :: components :: local_to_world ()) ? , translation : entity :: get_component (id , ambient_api :: core :: transform :: components :: translation ()) ? , rotation : entity :: get_component (id , ambient_api :: core :: transform :: components :: rotation ()) ? , health : entity :: get_component (id , crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: health ()) ? , max_health : entity :: get_component (id , crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health ()) ? , is_vehicle : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: is_vehicle ()) ? , last_distances : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_distances ()) ? , last_jump_time : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_jump_time ()) ? , last_slowdown_time : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_slowdown_time ()) ? , def_ref : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: def_ref ()) ? , input_direction : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_direction ()) ? , input_jump : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_jump ()) ? , input_fire : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_fire ()) ? , optional : VehicleOptional { driver_ref : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: driver_ref ()) , last_upside_down_time : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_upside_down_time ()) , aim_position : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: aim_position ()) , aimable_weapon_refs : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: aimable_weapon_refs ()) , } })
                }
                fn get_unspawned(entity: &Entity) -> Option<Self> {
                    Some (Self { linear_velocity : entity . get (ambient_api :: core :: physics :: components :: linear_velocity ()) ? , angular_velocity : entity . get (ambient_api :: core :: physics :: components :: angular_velocity ()) ? , physics_controlled : entity . get (ambient_api :: core :: physics :: components :: physics_controlled ()) ? , dynamic : entity . get (ambient_api :: core :: physics :: components :: dynamic ()) ? , density : entity . get (ambient_api :: core :: physics :: components :: density ()) ? , cube_collider : entity . get (ambient_api :: core :: physics :: components :: cube_collider ()) ? , local_to_world : entity . get (ambient_api :: core :: transform :: components :: local_to_world ()) ? , translation : entity . get (ambient_api :: core :: transform :: components :: translation ()) ? , rotation : entity . get (ambient_api :: core :: transform :: components :: rotation ()) ? , health : entity . get (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: health ()) ? , max_health : entity . get (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health ()) ? , is_vehicle : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: is_vehicle ()) ? , last_distances : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_distances ()) ? , last_jump_time : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_jump_time ()) ? , last_slowdown_time : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_slowdown_time ()) ? , def_ref : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: def_ref ()) ? , input_direction : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_direction ()) ? , input_jump : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_jump ()) ? , input_fire : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_fire ()) ? , optional : VehicleOptional { driver_ref : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: driver_ref ()) , last_upside_down_time : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_upside_down_time ()) , aim_position : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: aim_position ()) , aimable_weapon_refs : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: aimable_weapon_refs ()) , } })
                }
                fn contained_by_spawned(id: EntityId) -> bool {
                    entity :: has_components (id , & [& ambient_api :: core :: physics :: components :: linear_velocity () , & ambient_api :: core :: physics :: components :: angular_velocity () , & ambient_api :: core :: physics :: components :: physics_controlled () , & ambient_api :: core :: physics :: components :: dynamic () , & ambient_api :: core :: physics :: components :: density () , & ambient_api :: core :: physics :: components :: cube_collider () , & ambient_api :: core :: transform :: components :: local_to_world () , & ambient_api :: core :: transform :: components :: translation () , & ambient_api :: core :: transform :: components :: rotation () , & crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: health () , & crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: is_vehicle () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_distances () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_jump_time () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_slowdown_time () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: def_ref () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_direction () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_jump () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_fire ()])
                }
                fn contained_by_unspawned(entity: &Entity) -> bool {
                    entity . has_components (& [& ambient_api :: core :: physics :: components :: linear_velocity () , & ambient_api :: core :: physics :: components :: angular_velocity () , & ambient_api :: core :: physics :: components :: physics_controlled () , & ambient_api :: core :: physics :: components :: dynamic () , & ambient_api :: core :: physics :: components :: density () , & ambient_api :: core :: physics :: components :: cube_collider () , & ambient_api :: core :: transform :: components :: local_to_world () , & ambient_api :: core :: transform :: components :: translation () , & ambient_api :: core :: transform :: components :: rotation () , & crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: health () , & crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: is_vehicle () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_distances () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_jump_time () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_slowdown_time () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: def_ref () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_direction () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_jump () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_fire ()])
                }
            }
            impl ConceptComponents for Vehicle {
                type Required = (
                    Component<Vec3>,
                    Component<Vec3>,
                    Component<()>,
                    Component<bool>,
                    Component<f32>,
                    Component<Vec3>,
                    Component<Mat4>,
                    Component<Vec3>,
                    Component<Quat>,
                    Component<f32>,
                    Component<f32>,
                    Component<()>,
                    Component<Vec<f32>>,
                    Component<Duration>,
                    Component<Duration>,
                    Component<EntityId>,
                    Component<Vec2>,
                    Component<bool>,
                    Component<bool>,
                );
                type Optional = (
                    Component<EntityId>,
                    Component<Duration>,
                    Component<Vec3>,
                    Component<Vec<EntityId>>,
                );
                fn required() -> Self::Required {
                    (ambient_api :: core :: physics :: components :: linear_velocity () , ambient_api :: core :: physics :: components :: angular_velocity () , ambient_api :: core :: physics :: components :: physics_controlled () , ambient_api :: core :: physics :: components :: dynamic () , ambient_api :: core :: physics :: components :: density () , ambient_api :: core :: physics :: components :: cube_collider () , ambient_api :: core :: transform :: components :: local_to_world () , ambient_api :: core :: transform :: components :: translation () , ambient_api :: core :: transform :: components :: rotation () , crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: health () , crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: is_vehicle () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_distances () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_jump_time () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_slowdown_time () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: def_ref () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_direction () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_jump () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_fire () ,)
                }
                fn optional() -> Self::Optional {
                    (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: driver_ref () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_upside_down_time () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: aim_position () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: aimable_weapon_refs () ,)
                }
                fn from_required_data(required: <Self::Required as ComponentsTuple>::Data) -> Self {
                    Self {
                        linear_velocity: required.0,
                        angular_velocity: required.1,
                        physics_controlled: required.2,
                        dynamic: required.3,
                        density: required.4,
                        cube_collider: required.5,
                        local_to_world: required.6,
                        translation: required.7,
                        rotation: required.8,
                        health: required.9,
                        max_health: required.10,
                        is_vehicle: required.11,
                        last_distances: required.12,
                        last_jump_time: required.13,
                        last_slowdown_time: required.14,
                        def_ref: required.15,
                        input_direction: required.16,
                        input_jump: required.17,
                        input_fire: required.18,
                        optional: Default::default(),
                    }
                }
            }
            #[doc = "**Spawnpoint**: A spawnpoint\n\n**Required**:\n- `is_spawnpoint`: Is a spawnpoint\n- `radius`: Radius of the spawnpoint\n- `translation`: The translation/position of this entity.\n- `color`: This entity will be tinted with the specified color if the color is not black."]
            #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct Spawnpoint {
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::spawnpoint::is_spawnpoint`\n\n**Component description**: Is a spawnpoint\n\n"]
                pub is_spawnpoint: (),
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::spawnpoint::radius`\n\n**Component description**: Radius of the spawnpoint\n\n"]
                pub radius: f32,
                #[doc = "**Component**: `ambient_core::transform::translation`\n\n**Component description**: The translation/position of this entity.\n\n"]
                pub translation: Vec3,
                #[doc = "**Component**: `ambient_core::rendering::color`\n\n**Component description**: This entity will be tinted with the specified color if the color is not black.\n\n"]
                pub color: Vec4,
            }
            impl Concept for Spawnpoint {
                fn make(self) -> Entity {
                    let mut entity = Entity :: new () . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: spawnpoint :: components :: is_spawnpoint () , self . is_spawnpoint) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: spawnpoint :: components :: radius () , self . radius) . with (ambient_api :: core :: transform :: components :: translation () , self . translation) . with (ambient_api :: core :: rendering :: components :: color () , self . color) ;
                    entity
                }
                fn get_spawned(id: EntityId) -> Option<Self> {
                    Some (Self { is_spawnpoint : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: spawnpoint :: components :: is_spawnpoint ()) ? , radius : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: spawnpoint :: components :: radius ()) ? , translation : entity :: get_component (id , ambient_api :: core :: transform :: components :: translation ()) ? , color : entity :: get_component (id , ambient_api :: core :: rendering :: components :: color ()) ? , })
                }
                fn get_unspawned(entity: &Entity) -> Option<Self> {
                    Some (Self { is_spawnpoint : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: spawnpoint :: components :: is_spawnpoint ()) ? , radius : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: spawnpoint :: components :: radius ()) ? , translation : entity . get (ambient_api :: core :: transform :: components :: translation ()) ? , color : entity . get (ambient_api :: core :: rendering :: components :: color ()) ? , })
                }
                fn contained_by_spawned(id: EntityId) -> bool {
                    entity :: has_components (id , & [& crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: spawnpoint :: components :: is_spawnpoint () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: spawnpoint :: components :: radius () , & ambient_api :: core :: transform :: components :: translation () , & ambient_api :: core :: rendering :: components :: color ()])
                }
                fn contained_by_unspawned(entity: &Entity) -> bool {
                    entity . has_components (& [& crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: spawnpoint :: components :: is_spawnpoint () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: spawnpoint :: components :: radius () , & ambient_api :: core :: transform :: components :: translation () , & ambient_api :: core :: rendering :: components :: color ()])
                }
            }
            impl ConceptComponents for Spawnpoint {
                type Required = (
                    Component<()>,
                    Component<f32>,
                    Component<Vec3>,
                    Component<Vec4>,
                );
                type Optional = ();
                fn required() -> Self::Required {
                    (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: spawnpoint :: components :: is_spawnpoint () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: spawnpoint :: components :: radius () , ambient_api :: core :: transform :: components :: translation () , ambient_api :: core :: rendering :: components :: color () ,)
                }
                fn optional() -> Self::Optional {
                    ()
                }
                fn from_required_data(required: <Self::Required as ComponentsTuple>::Data) -> Self {
                    Self {
                        is_spawnpoint: required.0,
                        radius: required.1,
                        translation: required.2,
                        color: required.3,
                    }
                }
            }
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
    pub mod ambient_core {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("ambient_core")
                    .expect("Failed to get package entity - was it despawned?")
            });
            *ENTITY
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
    pub mod xadcwjtmzuagnoydry5h4psaph3hccbw {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("xadcwjtmzuagnoydry5h4psaph3hccbw")
                    .expect("Failed to get package entity - was it despawned?")
            });
            *ENTITY
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
    pub mod afl5yv5ya35vbuaj3aido22cwjzat25z {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("afl5yv5ya35vbuaj3aido22cwjzat25z")
                    .expect("Failed to get package entity - was it despawned?")
            });
            *ENTITY
        }
        #[doc = r" Auto-generated component definitions."]
        pub mod components {
            use ambient_api::{
                ecs::{Component, __internal_get_component},
                once_cell::sync::Lazy,
                prelude::*,
            };
            static RUN_DIRECTION: Lazy<Component<Vec2>> = Lazy::new(|| {
                __internal_get_component("afl5yv5ya35vbuaj3aido22cwjzat25z::run_direction")
            });
            #[doc = "**run_direction**\n\n*Attributes*: Debuggable, Networked"]
            pub fn run_direction() -> Component<Vec2> {
                *RUN_DIRECTION
            }
            static SPEED: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("afl5yv5ya35vbuaj3aido22cwjzat25z::speed"));
            #[doc = "**speed**: The speed the unit can walk at\n\n*Attributes*: Debuggable, Networked"]
            pub fn speed() -> Component<f32> {
                *SPEED
            }
            static RUN_SPEED_MULTIPLIER: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("afl5yv5ya35vbuaj3aido22cwjzat25z::run_speed_multiplier")
            });
            #[doc = "**run_speed_multiplier**: The speed the unit can run at\n\n*Attributes*: Debuggable, Networked"]
            pub fn run_speed_multiplier() -> Component<f32> {
                *RUN_SPEED_MULTIPLIER
            }
            static STRAFE_SPEED_MULTIPLIER: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component(
                    "afl5yv5ya35vbuaj3aido22cwjzat25z::strafe_speed_multiplier",
                )
            });
            #[doc = "**strafe_speed_multiplier**: The speed the unit can strafe at\n\n*Attributes*: Debuggable, Networked"]
            pub fn strafe_speed_multiplier() -> Component<f32> {
                *STRAFE_SPEED_MULTIPLIER
            }
            static AIR_SPEED_MULTIPLIER: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("afl5yv5ya35vbuaj3aido22cwjzat25z::air_speed_multiplier")
            });
            #[doc = "**air_speed_multiplier**: When the unit is in the air; how much can it control its movement? If this is 0, it can't control it at all. If it's 1 it's the same as on the ground.\n\n*Attributes*: Debuggable, Networked"]
            pub fn air_speed_multiplier() -> Component<f32> {
                *AIR_SPEED_MULTIPLIER
            }
            static UNIT_DISPLACEMENT: Lazy<Component<Vec3>> = Lazy::new(|| {
                __internal_get_component("afl5yv5ya35vbuaj3aido22cwjzat25z::unit_displacement")
            });
            #[doc = "**unit_displacement**: The distance the unit tried to move last frame (though it may have collided so the actual distance may be shorter).\n\n*Attributes*: Debuggable, Networked"]
            pub fn unit_displacement() -> Component<Vec3> {
                *UNIT_DISPLACEMENT
            }
            static JUMPING: Lazy<Component<bool>> =
                Lazy::new(|| __internal_get_component("afl5yv5ya35vbuaj3aido22cwjzat25z::jumping"));
            #[doc = "**jumping**\n\n*Attributes*: Debuggable, Networked"]
            pub fn jumping() -> Component<bool> {
                *JUMPING
            }
            static RUNNING: Lazy<Component<bool>> =
                Lazy::new(|| __internal_get_component("afl5yv5ya35vbuaj3aido22cwjzat25z::running"));
            #[doc = "**running**\n\n*Attributes*: Debuggable, Networked"]
            pub fn running() -> Component<bool> {
                *RUNNING
            }
            static SHOOTING: Lazy<Component<bool>> = Lazy::new(|| {
                __internal_get_component("afl5yv5ya35vbuaj3aido22cwjzat25z::shooting")
            });
            #[doc = "**shooting**\n\n*Attributes*: Debuggable, Networked"]
            pub fn shooting() -> Component<bool> {
                *SHOOTING
            }
            static VERTICAL_VELOCITY: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("afl5yv5ya35vbuaj3aido22cwjzat25z::vertical_velocity")
            });
            #[doc = "**Unit vertical speed**: The units's vertical speed.\n\n*Attributes*: Debuggable, Networked"]
            pub fn vertical_velocity() -> Component<f32> {
                *VERTICAL_VELOCITY
            }
            static IS_ON_GROUND: Lazy<Component<bool>> = Lazy::new(|| {
                __internal_get_component("afl5yv5ya35vbuaj3aido22cwjzat25z::is_on_ground")
            });
            #[doc = "**is_on_ground**\n\n*Attributes*: Debuggable, Networked"]
            pub fn is_on_ground() -> Component<bool> {
                *IS_ON_GROUND
            }
            static HEAD_REF: Lazy<Component<EntityId>> = Lazy::new(|| {
                __internal_get_component("afl5yv5ya35vbuaj3aido22cwjzat25z::head_ref")
            });
            #[doc = "**head_ref**\n\n*Attributes*: Debuggable, Networked"]
            pub fn head_ref() -> Component<EntityId> {
                *HEAD_REF
            }
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
    pub mod hvxms7i2px7krvkm23sxfjxsjqlcmtb5 {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("hvxms7i2px7krvkm23sxfjxsjqlcmtb5")
                    .expect("Failed to get package entity - was it despawned?")
            });
            *ENTITY
        }
        pub mod player {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use ambient_api::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static CONTROL_OF_ENTITY: Lazy<Component<EntityId>> = Lazy::new(|| {
                    __internal_get_component(
                        "hvxms7i2px7krvkm23sxfjxsjqlcmtb5::player::control_of_entity",
                    )
                });
                #[doc = "**Player's Control-of Entity**: The entity that this player is controlling.\n\n*Attributes*: Debuggable, Networked"]
                pub fn control_of_entity() -> Component<EntityId> {
                    *CONTROL_OF_ENTITY
                }
            }
        }
        #[doc = r" Auto-generated component definitions."]
        pub mod components {
            use ambient_api::{
                ecs::{Component, __internal_get_component},
                once_cell::sync::Lazy,
                prelude::*,
            };
            static HEALTH: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("hvxms7i2px7krvkm23sxfjxsjqlcmtb5::health"));
            #[doc = "**Health**: This game object's health. \"Standard\" health is 100 HP.\n\n*Attributes*: Debuggable, Networked"]
            pub fn health() -> Component<f32> {
                *HEALTH
            }
            static MAX_HEALTH: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("hvxms7i2px7krvkm23sxfjxsjqlcmtb5::max_health")
            });
            #[doc = "**Max Health**: Maximum health of the object. 100 HP is \"standard.\"\n\n*Attributes*: Debuggable, Networked"]
            pub fn max_health() -> Component<f32> {
                *MAX_HEALTH
            }
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
}
pub use raw::hvxms7i2px7krvkm23sxfjxsjqlcmtb5 as game_object;
pub use raw::mwrcsok65na7owrbdococ5sthr3ccskc as tangent_schema;
pub use raw::xadcwjtmzuagnoydry5h4psaph3hccbw as this;
