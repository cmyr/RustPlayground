//
//  XiApp.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-03-21.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

class XiApp: NSApplication {
    let handler = EventHandler(callback: dispatchEvent, action: handleAction)

    override func sendEvent(_ event: NSEvent) {
        if event.type == .keyDown {
            handler.handleInput(event: event)
        }
    }

    func reallySendEvent(_ event: NSEvent) {
        event.window?.sendEvent(event)
    }
}

func dispatchEvent(eventPtr: OpaquePointer?) {
    if let ptr = UnsafeRawPointer(eventPtr) {
        let event: NSEvent = Unmanaged<NSEvent>.fromOpaque(ptr).takeRetainedValue();
        print("dispatchEvent \(event.getAddress())")
        (NSApp as! XiApp).reallySendEvent(event)
    }
}

func handleAction(jsonPtr: UnsafePointer<Int8>?) {
    if let ptr = jsonPtr {
        let string = String(cString: ptr)

        let message = try! JSONSerialization.jsonObject(with: string.data(using: .utf8)!) as! [String: AnyObject]
        let method = message["method"] as! String
        let params = message["params"] as! [String: AnyObject]

        (NSApp.delegate as! AppDelegate).handleMessage(method: method, params: params)
    }
}
