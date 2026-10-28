/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Early UIKit toolbar items. Based on Apple's public UIBarButtonItem docs.

use crate::abi::GuestArg;
use crate::frameworks::core_graphics::CGFloat;
use crate::frameworks::foundation::ns_string::get_static_str;
use crate::frameworks::foundation::NSInteger;
use crate::frameworks::uikit::ui_view::ui_control::UIControlEventTouchUpInside;
use crate::objc::{
    id, msg, msg_class, msg_send, msg_super, nil, objc_classes, release, retain, ClassExports,
    HostObject, NSZonePtr, SEL,
};
use crate::Environment;

pub(super) struct ItemHostObject {
    pub(super) button: id,
    pub(super) system: Option<NSInteger>,
    pub(super) width: CGFloat,
    title: id,
    target: id, // weak
    action: Option<SEL>,
    style: NSInteger,
    enabled: bool,
}
impl HostObject for ItemHostObject {}
impl Default for ItemHostObject {
    fn default() -> Self {
        Self {
            button: nil,
            system: None,
            width: 0.0,
            title: nil,
            target: nil,
            action: None,
            style: 0,
            enabled: true,
        }
    }
}

fn make_button(env: &mut Environment, this: id) {
    let button: id = msg_class![env; UIButton new];
    let action = env
        .objc
        .register_host_selector("_touchHLE_activate".into(), &mut env.mem);
    () = msg![env; button addTarget:this action:action
        forControlEvents:UIControlEventTouchUpInside];
    env.objc.borrow_mut::<ItemHostObject>(this).button = button;
}

pub const CLASSES: ClassExports = objc_classes! {
(env, this, _cmd);

@implementation UIBarItem: NSObject
@end

@implementation UIBarButtonItem: UIBarItem

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(this, Box::<ItemHostObject>::default(), &mut env.mem)
}
- (id)init {
    make_button(env, this);
    this
}
- (id)initWithTitle:(id)title style:(NSInteger)style
              target:(id)target action:(SEL)action {
    let this: id = msg![env; this init];
    () = msg![env; this setTitle:title];
    let item = env.objc.borrow_mut::<ItemHostObject>(this);
    item.style = style;
    item.target = target;
    item.action = Some(action);
    this
}
- (id)initWithBarButtonSystemItem:(NSInteger)system
                          target:(id)target action:(SEL)action {
    let this: id = msg![env; this init];
    let title = match system {
        0 => "Done", 1 => "Cancel", 2 => "Edit", 3 => "Save", 4 => "+",
        5 | 6 => "", 7 => "Compose", 8 => "Reply", 9 => "Action",
        10 => "Organize", 11 => "Bookmarks", 12 => "Search",
        13 => "Refresh", 14 => "Stop", 15 => "Camera", 16 => "Trash",
        17 => "Play", 18 => "Pause", 19 => "Rewind", 20 => "Fast forward",
        _ => { log!("TODO: toolbar system item {}", system); "?" }
    };
    let title = get_static_str(env, title);
    () = msg![env; this setTitle:title];
    let item = env.objc.borrow_mut::<ItemHostObject>(this);
    item.system = Some(system);
    item.target = target;
    item.action = Some(action);
    this
}
- (())dealloc {
    let item = std::mem::take(env.objc.borrow_mut::<ItemHostObject>(this));
    release(env, item.title);
    release(env, item.button);
    msg_super![env; this dealloc]
}
- (id)title { env.objc.borrow::<ItemHostObject>(this).title }
- (())setTitle:(id)title {
    let title: id = msg![env; title copy];
    let item = env.objc.borrow_mut::<ItemHostObject>(this);
    let old = std::mem::replace(&mut item.title, title);
    let button = item.button;
    release(env, old);
    () = msg![env; button setTitle:title forState:0u32];
}
- (CGFloat)width { env.objc.borrow::<ItemHostObject>(this).width }
- (())setWidth:(CGFloat)width {
    env.objc.borrow_mut::<ItemHostObject>(this).width = width.max(0.0);
}
- (NSInteger)style { env.objc.borrow::<ItemHostObject>(this).style }
- (())setStyle:(NSInteger)style { env.objc.borrow_mut::<ItemHostObject>(this).style = style; }
- (id)target { env.objc.borrow::<ItemHostObject>(this).target }
- (())setTarget:(id)target { env.objc.borrow_mut::<ItemHostObject>(this).target = target; }
- (SEL)action {
    env.objc.borrow::<ItemHostObject>(this).action.unwrap_or_else(||
        <SEL as GuestArg>::from_regs(&[0]))
}
- (())setAction:(SEL)action { env.objc.borrow_mut::<ItemHostObject>(this).action = Some(action); }
- (bool)isEnabled { env.objc.borrow::<ItemHostObject>(this).enabled }
- (())setEnabled:(bool)enabled {
    let item = env.objc.borrow_mut::<ItemHostObject>(this);
    item.enabled = enabled;
    let button = item.button;
    () = msg![env; button setEnabled:enabled];
}
- (())_touchHLE_activate {
    let item = env.objc.borrow::<ItemHostObject>(this);
    let (target, action, enabled) = (item.target, item.action, item.enabled);
    if !enabled { return; }
    if let Some(action) = action.filter(|action| !action.is_null()) {
        if target == nil {
            log!("TODO: toolbar action via responder chain");
            return;
        }
        // The sender is the bar item, not its backing UIButton.
        let count = action.as_str(&env.mem).bytes().filter(|b| *b == b':').count();
        retain(env, this);
        match count {
            0 => { () = msg_send(env, (target, action)); }
            1 => { () = msg_send(env, (target, action, this)); }
            _ => { log!("Unsupported toolbar action {:?}", action); }
        }
        release(env, this);
    }
}
@end
};
