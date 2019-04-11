//
//  Style.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-04-11.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

typealias StyleId = UInt32

class StyleMap {
    private var inner: [StyleId: Style] = [:]

    func addStyle(withId styleId: StyleId, style: Style) {
        print("adding style \(styleId): \(style)")
        inner[styleId] = style
    }

    func style(forId styleId: StyleId) -> Style {
        let style = inner[styleId]
        assert(style != nil, "it is an invariant that styles are defined before use")
        return style!
    }
}

struct Style {
    let foreground: NSColor
    let background: NSColor
    let italic: Bool
    let bold: Bool
    let underline: Bool

    static func fromJson(_ json: [String: AnyObject]) -> Style {
        let foreground = json["foreground"] as! UInt32
        let background = json["background"] as! UInt32
        let italic = json["italic"] as! Bool
        let bold = json["bold"] as! Bool
        let underline = json["underline"] as! Bool
        return Style(foreground: NSColor.fromArgb(foreground),
              background: NSColor.fromArgb(background),
              italic: italic,
              bold: bold,
              underline: underline)
    }
}

extension NSColor {
    static func fromArgb(_ argb: UInt32) -> NSColor {
        return NSColor(red: CGFloat((argb >> 16) & 0xff) * 1.0/255,
                       green: CGFloat((argb >> 8) & 0xff) * 1.0/255,
                       blue: CGFloat(argb & 0xff) * 1.0/255,
                       alpha: CGFloat((argb >> 24) & 0xff) * 1.0/255)
    }
}
