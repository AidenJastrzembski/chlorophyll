use crate::Theme;
// TODO: these should be defined by the user in their config, or better yet
// just pass the wallpaper in at runtime and let the cli handle everything else

pub const ZEN: Theme = Theme {
    wallpaper: "zen.jpg",
    is_animated: false,
};

pub const HIDEOUT: Theme = Theme {
    wallpaper: "hideout.png",
    is_animated: false,
};

pub const FREAK: Theme = Theme {
    wallpaper: "freak.jpg",
    is_animated: false,
};

pub const BLEAK: Theme = Theme {
    wallpaper: "bleak.gif",
    is_animated: true,
};
