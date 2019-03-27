//
//  XiEventHandler.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-03-18.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

class EventHandler {
    let _inner: OpaquePointer

    init(callback: @escaping (@convention(c) (OpaquePointer?, Bool) -> Void),
         action: @escaping (@convention(c) (UnsafePointer<Int8>?) -> Void),
         timer: @escaping (@convention(c) (OpaquePointer?, UInt32) -> UInt32),
         cancelTimer: @escaping (@convention(c) (UInt32) -> Void)
         ) {
        _inner = xiEventHandlerCreate(callback, action, timer, cancelTimer)
    }

    func handleInput(event: NSEvent) {
        var chars = event.charactersIgnoringModifiers ?? "";

        switch event.keyCode {
        case 53:
            chars = "Escape"
        case 51:
            chars = "Backspace"
        case 117:
            chars = "Delete"
        case 76:
            chars = "Enter"
        case 123:
            chars = "LeftArrow"
        case 124:
            chars = "RightArrow"
        case 125:
            chars = "DownArrow"
        case 126:
            chars = "UpArrow"
        default: break
        }

        let modifiers = UInt32(event.modifierFlags.rawValue);
        let eventPtr: Unmanaged<NSEvent> = Unmanaged.passRetained(event);

        print("sending \(event.getAddress()) \(event)")
        xiEventHandlerHandleInput(_inner, modifiers, chars, OpaquePointer(eventPtr.toOpaque()))
    }

    func clearPending(_ identifier: UInt32) {
        xiEventHandlerClearPending(_inner, identifier)
    }

    deinit {
        xiEventHandlerFree(_inner)
    }
}

extension NSEvent {
    func getAddress() -> String {
        return Unmanaged.passUnretained(self).toOpaque().debugDescription
    }
}
