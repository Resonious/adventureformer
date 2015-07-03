use std::mem::zeroed;
use render::{ImageAsset, Texcoords};

macro_rules! image_assets {
    ($($texcoords_name:ident $name:ident: $sprite_type:ty [$texcoords:expr][$w:expr;$h:expr] $path:expr),+) =>  {

    pub struct Images {
        $(
        pub $name: ImageAsset,
        pub $texcoords_name: [Texcoords; $texcoords]
        // concat_idents!($name, _texcoords): [Texcoords, $num_texcoords]
        ),*
    }

    impl Images {
        pub fn init(&mut self) {
            $(
            self.$name = ImageAsset {
                filename: $path,
                vbo: 0,
                texture: unsafe { zeroed() },
                frame_width: $w,
                frame_height: $h,
                texcoord_count: $texcoords
            };
            )*
        }
    }
    }
}

image_assets!(
    ccbdy crattlecrute_body:       SpriteType1 [9][90;90] "assets/crattlecrute/body.png",
    ccbft crattlecrute_back_foot:  SpriteType1 [9][90;90] "assets/crattlecrute/back-foot.png",
    ccfft crattlecrute_front_foot: SpriteType1 [9][90;90] "assets/crattlecrute/front-foot.png"
);
