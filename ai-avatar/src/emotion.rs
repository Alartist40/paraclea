//! Emotion state machine and avatar parameter mapping.
//!
//! Based on Plutchik's wheel of emotions, extended with derived expressions.


/// Primary and derived emotions in the Plutchik model.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EmotionState {
    // Primary emotions (0.0 – 1.0)
    pub joy: f32,
    pub trust: f32,
    pub fear: f32,
    pub surprise: f32,
    pub sadness: f32,
    pub disgust: f32,
    pub anger: f32,
    pub anticipation: f32,

    // Derived expressions
    pub love: f32,
    pub submission: f32,
    pub awe: f32,
    pub disapproval: f32,
    pub remorse: f32,
    pub contempt: f32,
    pub aggressiveness: f32,
    pub optimism: f32,
}

impl EmotionState {
    /// Recompute derived emotions from primaries.
    pub fn recompute_derived(&mut self) {
        self.love = (self.joy + self.trust) * 0.5;
        self.submission = (self.trust + self.fear) * 0.5;
        self.awe = (self.fear + self.surprise) * 0.5;
        self.disapproval = (self.surprise + self.sadness) * 0.5;
        self.remorse = (self.sadness + self.disgust) * 0.5;
        self.contempt = (self.disgust + self.anger) * 0.5;
        self.aggressiveness = (self.anger + self.anticipation) * 0.5;
        self.optimism = (self.anticipation + self.joy) * 0.5;
    }

    /// Linear interpolation between two emotion states.
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        macro_rules! lerp_field {
            ($field:ident) => {
                self.$field + (other.$field - self.$field) * t
            };
        }
        let mut state = Self {
            joy: lerp_field!(joy),
            trust: lerp_field!(trust),
            fear: lerp_field!(fear),
            surprise: lerp_field!(surprise),
            sadness: lerp_field!(sadness),
            disgust: lerp_field!(disgust),
            anger: lerp_field!(anger),
            anticipation: lerp_field!(anticipation),
            love: lerp_field!(love),
            submission: lerp_field!(submission),
            awe: lerp_field!(awe),
            disapproval: lerp_field!(disapproval),
            remorse: lerp_field!(remorse),
            contempt: lerp_field!(contempt),
            aggressiveness: lerp_field!(aggressiveness),
            optimism: lerp_field!(optimism),
        };
        state.recompute_derived();
        state
    }

    /// Convert emotion state to avatar morph-target parameters.
    pub fn to_avatar_params(&self) -> AvatarParams {
        AvatarParams {
            brow_raise: self.joy * 0.8 + self.surprise * 1.0,
            brow_furrow: self.anger * 0.8 + self.disgust * 0.5,
            mouth_smile: self.joy * 1.0 + self.optimism * 0.5,
            mouth_frown: self.sadness * 0.8 + self.remorse * 0.4,
            mouth_open: self.surprise * 0.7 + self.fear * 0.3,
            eye_wide: self.surprise * 0.8 + self.joy * 0.3,
            eye_squint: self.anger * 0.5 + self.joy * 0.2,
            blush: self.love * 0.8 + self.submission * 0.4,
            head_tilt: (self.trust - self.fear) * 15.0,
            head_bob: self.joy * 0.3 + self.excitement() * 0.5,
            excitement: self.excitement(),
        }
    }

    /// Overall excitement level.
    pub fn excitement(&self) -> f32 {
        (self.joy + self.anticipation + self.surprise) / 3.0
    }
}

/// Parameters that drive the avatar's morph targets.
/// These are generic names that map to Inochi2D / Live2D parameters.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AvatarParams {
    pub brow_raise: f32,
    pub brow_furrow: f32,
    pub mouth_smile: f32,
    pub mouth_frown: f32,
    pub mouth_open: f32,
    pub eye_wide: f32,
    pub eye_squint: f32,
    pub blush: f32,
    pub head_tilt: f32,
    pub head_bob: f32,
    pub excitement: f32,
}

/// A simple spring-damper for smoothing scalar values over time.
pub struct Smoother {
    current: f32,
    target: f32,
    velocity: f32,
    tension: f32,
    damping: f32,
}

impl Smoother {
    pub fn new(value: f32, tension: f32, damping: f32) -> Self {
        Self {
            current: value,
            target: value,
            velocity: 0.0,
            tension,
            damping,
        }
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    pub fn update(&mut self, dt: f32) {
        let force = self.tension * (self.target - self.current);
        self.velocity += force * dt;
        self.velocity *= 1.0 - self.damping * dt;
        self.current += self.velocity * dt;
    }

    pub fn value(&self) -> f32 {
        self.current
    }
}

/// Smoothly interpolate an entire `AvatarParams` struct toward a target.
pub struct AvatarSmoother {
    pub brow_raise: Smoother,
    pub brow_furrow: Smoother,
    pub mouth_smile: Smoother,
    pub mouth_frown: Smoother,
    pub mouth_open: Smoother,
    pub eye_wide: Smoother,
    pub eye_squint: Smoother,
    pub blush: Smoother,
    pub head_tilt: Smoother,
    pub head_bob: Smoother,
    pub excitement: Smoother,
}

impl AvatarSmoother {
    pub fn new(tension: f32, damping: f32) -> Self {
        Self {
            brow_raise: Smoother::new(0.0, tension, damping),
            brow_furrow: Smoother::new(0.0, tension, damping),
            mouth_smile: Smoother::new(0.0, tension, damping),
            mouth_frown: Smoother::new(0.0, tension, damping),
            mouth_open: Smoother::new(0.0, tension, damping),
            eye_wide: Smoother::new(0.0, tension, damping),
            eye_squint: Smoother::new(0.0, tension, damping),
            blush: Smoother::new(0.0, tension, damping),
            head_tilt: Smoother::new(0.0, tension, damping),
            head_bob: Smoother::new(0.0, tension, damping),
            excitement: Smoother::new(0.0, tension, damping),
        }
    }

    pub fn set_target(&mut self, params: &AvatarParams) {
        self.brow_raise.set_target(params.brow_raise);
        self.brow_furrow.set_target(params.brow_furrow);
        self.mouth_smile.set_target(params.mouth_smile);
        self.mouth_frown.set_target(params.mouth_frown);
        self.mouth_open.set_target(params.mouth_open);
        self.eye_wide.set_target(params.eye_wide);
        self.eye_squint.set_target(params.eye_squint);
        self.blush.set_target(params.blush);
        self.head_tilt.set_target(params.head_tilt);
        self.head_bob.set_target(params.head_bob);
        self.excitement.set_target(params.excitement);
    }

    pub fn update(&mut self, dt: f32) {
        self.brow_raise.update(dt);
        self.brow_furrow.update(dt);
        self.mouth_smile.update(dt);
        self.mouth_frown.update(dt);
        self.mouth_open.update(dt);
        self.eye_wide.update(dt);
        self.eye_squint.update(dt);
        self.blush.update(dt);
        self.head_tilt.update(dt);
        self.head_bob.update(dt);
        self.excitement.update(dt);
    }

    pub fn current(&self) -> AvatarParams {
        AvatarParams {
            brow_raise: self.brow_raise.value(),
            brow_furrow: self.brow_furrow.value(),
            mouth_smile: self.mouth_smile.value(),
            mouth_frown: self.mouth_frown.value(),
            mouth_open: self.mouth_open.value(),
            eye_wide: self.eye_wide.value(),
            eye_squint: self.eye_squint.value(),
            blush: self.blush.value(),
            head_tilt: self.head_tilt.value(),
            head_bob: self.head_bob.value(),
            excitement: self.excitement.value(),
        }
    }
}
