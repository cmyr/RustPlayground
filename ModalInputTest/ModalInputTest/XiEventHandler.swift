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


class XiCore {
    let _inner: OpaquePointer

    init(rpcCallback: @escaping (@convention(c) (UnsafePointer<Int8>?) -> Void),
         updateCallback: @escaping (@convention(c) (UInt32) -> Void)
        ) {
        _inner = xiCoreCreate(rpcCallback, updateCallback)
    }

    func insertText(_ text: String) {
        xiCoreSendMessage(_inner, "insert \(text)")
    }

    func doCommand(_ command: String) {
        xiCoreSendMessage(_inner, command)
    }

    func getLine(_ lineNumber: UInt32) -> RawLine? {
        let line = xiCoreGetLine(_inner, lineNumber);
        if let line =  line {
            let string =  String(cString: line.pointee.text, encoding: .utf8)!
            let cursor: Int? = line.pointee.cursor < 0 ? nil : Int(line.pointee.cursor)
            return RawLine(text: string, cursor: cursor)
        } else {
            return nil
        }
    }

    deinit {
        xiCoreFree(_inner)
    }
}

class RawLine {
    let text: String
    let cursor: Int?

    init(text: String, cursor: Int?) {
        self.text = text
        self.cursor = cursor
    }
}

extension NSEvent {
    func getAddress() -> String {
        return Unmanaged.passUnretained(self).toOpaque().debugDescription
    }
}
