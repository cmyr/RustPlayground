//
//  XiWindow.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-03-22.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

class XiWindow: NSWindow {
    override func sendEvent(_ event: NSEvent) {
        if event.type == .keyDown {
            (NSApp.delegate as! AppDelegate).core.handleInput(event: event)
        } else {
            super.sendEvent(event)
        }
    }

    func reallySendEvent(_ event: NSEvent) {
        self.makeFirstResponder(self.contentViewController)
        super.sendEvent(event)
    }
}
