//! An app's title in the sidebar's title band (`SplitView::sidebar_title`): past the traffic
//! lights and the toggle, centred in the band, and gone with the sidebar's contents.
use rune::runtime::ElementInfo;
use rune::{Chrome, Cx, El, Env, Runtime, el};
use rune_kit::Kit;
use rune_kit::shell::{PaneState, SidebarToggle, SplitView};

struct S {
    sidebar: PaneState,
    in_header: bool,
}

fn view(s: &S, cx: &mut Cx<S>) -> El {
    let k = Kit::neutral(false);
    let toggle = if s.in_header { SidebarToggle::new().in_header() } else { SidebarToggle::new() };
    SplitView::new(el().key("body"))
        .sidebar(&s.sidebar, |s: &mut S| &mut s.sidebar, el().key("list"))
        .sidebar_toggle(toggle)
        .sidebar_title(el().key("app-title").size(60.0, 20.0))
        .titlebar_inset(cx.titlebar_inset())
        .build(&k)
}

fn find(rt: &Runtime<S>, key: &str) -> ElementInfo {
    rt.elements().into_iter().find(|e| e.key.as_deref() == Some(key) && !e.exiting).unwrap_or_else(|| panic!("no {key}"))
}

#[test]
fn the_sidebar_title_follows_the_toggle_in_the_title_band_and_closes_with_the_sidebar() {
    let inset = Chrome::DEFAULT.titlebar_inset();
    for in_header in [false, true] {
        let mut rt = Runtime::new(S { sidebar: PaneState::new(240.0), in_header }, view, (800.0, 500.0), Env::new(false, 1.0));
        rt.set_titlebar_inset(inset);
        rt.frame(0.0);
        let title = find(&rt, "app-title").rect;
        let toggle = find(&rt, "sidebar-toggle").rect;
        let band = find(&rt, "sidebar-titlebar").rect;
        assert_eq!(title[1] + title[3] / 2.0, band[1] + band[3] / 2.0, "in_header {in_header}: centred in the band");
        if in_header {
            assert_eq!(title[0], inset + 8.0, "past the lights");
            assert!(title[0] + title[2] <= toggle[0], "before the toggle at the band's end");
        } else {
            assert_eq!(title[0], toggle[0] + toggle[2] + 8.0, "past the toggle");
        }
        rt.update(|s, _| s.sidebar.toggle());
        for _ in 0..120 {
            rt.frame(1.0 / 60.0);
        }
        assert_eq!(find(&rt, "app-title").opacity, 0.0, "in_header {in_header}: it closes with the sidebar");
    }
}
