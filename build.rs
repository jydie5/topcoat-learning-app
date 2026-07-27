fn main() {
    topcoat::tailwind::BuildConfig::new()
        .input("styles.css")
        .render()
        .expect("Tailwind CSSの生成に失敗しました");

    topcoat::icon::iconify::BuildConfig::new()
        .icon_set("feather")
        .stage()
        .expect("Featherアイコンの準備に失敗しました");
}
