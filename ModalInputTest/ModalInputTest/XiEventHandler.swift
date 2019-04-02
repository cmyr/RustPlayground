//
//  XiEventHandler.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-03-18.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

class XiCore {
    let _inner: OpaquePointer
    private var _hasInputHandler = false
    var hasInputHandler: Bool {
        return _hasInputHandler
    }

    init(rpcCallback: @escaping (@convention(c) (UnsafePointer<Int8>?) -> Void),
         updateCallback: @escaping (@convention(c) (UInt32) -> Void)
        ) {
        _inner = xiCoreCreate(rpcCallback, updateCallback)
    }

    func registerEventHandler(callback: @escaping (@convention(c) (OpaquePointer?, Bool) -> Void),
         action: @escaping (@convention(c) (UnsafePointer<Int8>?) -> Void),
         timer: @escaping (@convention(c) (OpaquePointer?, UInt32) -> UInt32),
         cancelTimer: @escaping (@convention(c) (UInt32) -> Void)
        ) {
        guard !hasInputHandler else {
            fatalError("inputhandler can only be set up once")
        }
        _hasInputHandler = true
        xiCoreRegisterEventHandler(_inner, callback, action, timer, cancelTimer)
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
        xiCoreHandleInput(_inner, modifiers, chars, OpaquePointer(eventPtr.toOpaque()))
    }

    func clearPending(_ identifier: UInt32) {
        xiCoreClearPending(_inner, identifier)
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
            let selection: Range<Int>?
            if line.pointee.selection.0 == line.pointee.selection.1 {
                selection = nil
            } else {
                selection = Int(line.pointee.selection.0)..<Int(line.pointee.selection.1)
            }
            return RawLine(text: string, cursor: cursor, selection: selection)
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
    let selection: Range<Int>?

    init(text: String, cursor: Int?, selection: Range<Int>?) {
        self.text = text
        self.cursor = cursor
        self.selection = selection
    }
}

extension NSEvent {
    func getAddress() -> String {
        return Unmanaged.passUnretained(self).toOpaque().debugDescription
    }
}
