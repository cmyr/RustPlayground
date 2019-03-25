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
        //FIXME: hack to send escape
        if event.keyCode == 53 {
            chars = "␛"
        }

        let modifiers = UInt32(event.modifierFlags.rawValue);
        let eventPtr: Unmanaged<NSEvent> = Unmanaged.passRetained(event);

        print("sending \(event.getAddress()) \(event)")
        xiEventHandlerHandleInput(_inner, modifiers, chars, OpaquePointer(eventPtr.toOpaque()))
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
