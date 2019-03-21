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

    init(callback: @escaping (@convention(c) (OpaquePointer?) -> Void)) {
        _inner = xiEventHandlerCreate(callback)
    }

    func handleInput(event: NSEvent) {
        let chars = event.charactersIgnoringModifiers ?? "";
        let charsPtr = UnsafePointer<Int8>(chars)
        let modifiers = UInt32(event.modifierFlags.rawValue);
        let eventPtr: Unmanaged<NSEvent> = Unmanaged.passRetained(event);

        print("sending \(event.getAddress()) \(event)")
        xiEventHandlerHandleInput(_inner, modifiers, charsPtr, OpaquePointer(eventPtr.toOpaque()))
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
