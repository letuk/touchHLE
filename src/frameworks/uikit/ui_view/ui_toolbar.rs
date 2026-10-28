/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Early UIKit toolbar support. Native gradients and animation remain TODO.

use super::UIViewHostObject;
use crate::frameworks::core_graphics::{CGPoint, CGRect, CGSize};
use crate::frameworks::foundation::{NSInteger, NSUInteger};
use crate::frameworks::uikit::ui_bar_button_item::ItemHostObject;
use crate::objc::{
    id, impl_HostObject_with_superclass, msg, msg_class, msg_super, nil, objc_classes, release,
    ClassExports, NSZonePtr,
};
use crate::Environment;

#[derive(Default)]
struct ToolbarHostObject {
    superclass: UIViewHostObject,
    items: id,
    style: NSInteger,
    translucent: bool,
}
impl_HostObject_with_superclass!(ToolbarHostObject);

fn items(env: &mut Environment, this: id) -> Vec<id> {
    let array = env.objc.borrow::<ToolbarHostObject>(this).items;
    if array == nil {
        return Vec::new();
    }
    let count: NSUInteger = msg![env; array count];
    (0..count)
        .map(|i| msg![env; array objectAtIndex:i])
        .collect()
}

fn layout(env: &mut Environment, this: id) {
    let bounds: CGRect = msg![env; this bounds];
    let entries = items(env, this);
    let mut fixed = 0.0f32;
    let mut flexible = 0;
    let widths: Vec<_> = entries
        .iter()
        .map(|&entry| {
            let item = env.objc.borrow::<ItemHostObject>(entry);
            let width = match item.system {
                Some(5) => {
                    flexible += 1;
                    0.0
                }
                Some(6) => item.width,
                _ if item.width > 0.0 => item.width,
                _ => 72.0,
            };
            fixed += width;
            width
        })
        .collect();
    let gap = if flexible > 0 {
        ((bounds.size.width - 12.0 - fixed) / flexible as f32).max(0.0)
    } else {
        0.0
    };
    let mut x = bounds.origin.x + 6.0;
    for (entry, mut width) in entries.into_iter().zip(widths) {
        let item = env.objc.borrow::<ItemHostObject>(entry);
        let button = item.button;
        let spacer = matches!(item.system, Some(5 | 6));
        if item.system == Some(5) {
            width = gap;
        }
        if !spacer {
            let frame = CGRect {
                origin: CGPoint {
                    x,
                    y: bounds.origin.y + 4.0,
                },
                size: CGSize {
                    width,
                    height: (bounds.size.height - 8.0).max(0.0),
                },
            };
            () = msg![env; button setFrame:frame];
        }
        x += width;
    }
}

pub const CLASSES: ClassExports = objc_classes! {
(env, this, _cmd);
@implementation UIToolbar: UIView
+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(this, Box::<ToolbarHostObject>::default(), &mut env.mem)
}
- (id)initWithFrame:(CGRect)frame {
    let this: id = msg_super![env; this initWithFrame:frame];
    () = msg![env; this setBarStyle:0i32];
    this
}
- (())dealloc {
    let array = std::mem::replace(&mut env.objc.borrow_mut::<ToolbarHostObject>(this).items, nil);
    release(env, array);
    msg_super![env; this dealloc]
}
- (id)items { env.objc.borrow::<ToolbarHostObject>(this).items }
- (())setItems:(id)array {
    () = msg![env; this setItems:array animated:false];
}
- (())setItems:(id)array animated:(bool)animated {
    if animated { log!("TODO: animated toolbar item changes"); }
    let copy: id = msg![env; array copy];
    for entry in items(env, this) {
        let button = env.objc.borrow::<ItemHostObject>(entry).button;
        () = msg![env; button removeFromSuperview];
    }
    let old = std::mem::replace(&mut env.objc.borrow_mut::<ToolbarHostObject>(this).items, copy);
    release(env, old);
    for entry in items(env, this) {
        let item = env.objc.borrow::<ItemHostObject>(entry);
        let (button, spacer) = (item.button, matches!(item.system, Some(5 | 6)));
        if !spacer { () = msg![env; this addSubview:button]; }
    }
    layout(env, this);
}
- (NSInteger)barStyle { env.objc.borrow::<ToolbarHostObject>(this).style }
- (())setBarStyle:(NSInteger)style {
    env.objc.borrow_mut::<ToolbarHostObject>(this).style = style;
    let color: id = if style == 0 { msg_class![env; UIColor grayColor] }
        else { msg_class![env; UIColor blackColor] };
    () = msg![env; this setBackgroundColor:color];
}
- (bool)isTranslucent { env.objc.borrow::<ToolbarHostObject>(this).translucent }
- (())setTranslucent:(bool)translucent {
    env.objc.borrow_mut::<ToolbarHostObject>(this).translucent = translucent;
    // Keep the toolbar opaque until compositing matches early UIKit.
    log!("TODO: toolbar translucency {}", translucent);
}
- (CGSize)sizeThatFits:(CGSize)size { CGSize { width: size.width, height: 44.0 } }
- (())sizeToFit {
    let mut frame: CGRect = msg![env; this frame];
    frame.size.height = 44.0;
    () = msg![env; this setFrame:frame];
}
- (())setFrame:(CGRect)frame {
    () = msg_super![env; this setFrame:frame];
    layout(env, this);
}
- (())layoutSubviews { layout(env, this); }
@end
};
