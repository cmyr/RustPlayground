//
//  XiWindow.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-03-22.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

class XiWindow: NSWindow {

    override init(contentRect: NSRect, styleMask style: NSWindow.StyleMask, backing backingStoreType: NSWindow.BackingStoreType, defer flag: Bool) {
        super.init(contentRect: contentRect, styleMask: style, backing: backingStoreType, defer: flag)
        styleMask = [styleMask, .fullSizeContentView]
    }

    override func sendEvent(_ event: NSEvent) {
        let core = (NSApp.delegate as! AppDelegate).core
        if event.type == .keyDown && core.hasInputHandler {
            core.handleInput(event: event)
        } else {
            super.sendEvent(event)
        }
    }

    func reallySendEvent(_ event: NSEvent) {
        self.makeFirstResponder(self.contentViewController)
        super.sendEvent(event)
    }
}
