//
//  XiEventHandler.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-03-18.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

class XiCoreProxy {
    let _inner: OpaquePointer
    private var _hasInputHandler = false
    var hasInputHandler: Bool {
        return _hasInputHandler
    }

    init(rpcCallback: @escaping (@convention(c) (UnsafePointer<Int8>?) -> Void),
         updateCallback: @escaping (@convention(c) (Int, Int) -> Void),
         widthMeasure: @escaping (@convention(c) (UnsafePointer<Int8>?) -> XiSize)
        ) {
        _inner = xiCoreCreate(rpcCallback, updateCallback, widthMeasure)
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
        sendRpc(method: "insert", params: ["chars": text])
    }

    func updateConfig(_ settings: [String: Any]) {
        sendRpc(method: "modify_user_config", params: ["domain": "general", "changes": settings])
    }

    func frameChanged(newFrame rect: CGRect) {
        let rect = rect.integral
        let params = ["x": rect.origin.x,
                      "y": rect.origin.y,
                      "width": rect.width,
                      "height": rect.height]
        sendRpc(method: "viewport_change", params: params)
    }

    func doGesture(position: BufferPosition, type: Any) {
        let params = ["line": position.line,
                      "col": position.column,
                      "ty": type]
        sendRpc(method: "gesture", params: params)
    }

    func doCommand(_ command: String) {
        sendRpc(method: command, params: [])
    }

    /// returns the total contents of the buffer
    func getDocument() -> String {
        // hack; we're overriding the getline API in core if passed this
        // special line number
        let magicNumber: UInt32 = 6942069
        return getLine(magicNumber)!.text
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
            assert(line.pointee.styles_len % 3 == 0, "styles are guaranteed to be triplets")
            let buffer = UnsafeBufferPointer(start: line.pointee.styles, count: line.pointee.styles_len);
            var i = 0
            var styles = [StyleSpan]()
            while i < buffer.count {
                let start = buffer[i]
                let len = buffer[i+1]
                let styleId = buffer[i+2]
                let style = StyleSpan(start: start, len: len, styleId: UInt32(styleId))
                styles.append(style)
                i += 3
            }
            let result = RawLine(text: string, cursor: cursor, selection: selection, styles: styles)
            xiLineFree(line)
            return result

        } else {
            return nil
        }
    }

    func sendRpc(method: String, params: Any) {
        let req: [String: Any] = ["method": method, "params": params]
        sendJson(req)
    }

    private func sendJson(_ json: Any) {
        do {
            let data = try JSONSerialization.data(withJSONObject: json, options: [])
            let string = String(data: data, encoding: .utf8)
            xiCoreSendMessage(_inner, string)
        } catch _ {
            print("error serializing to json")
        }
    }

    deinit {
        xiCoreFree(_inner)
    }
}

struct StyleSpan {
    let start: Int
    let len: Int
    let styleId: StyleId
}

class RawLine {
    let text: String
    let cursor: Int?
    let selection: Range<Int>?
    let styles: [StyleSpan]

    static func placeholder() -> RawLine {
        return RawLine(text: "PLACEHOLDER", cursor: nil, selection: nil, styles: [])
    }

    init(text: String, cursor: Int?, selection: Range<Int>?, styles: [StyleSpan]) {
        self.text = text
        self.cursor = cursor
        self.selection = selection
        self.styles = styles
    }
}

extension NSEvent {
    func getAddress() -> String {
        return Unmanaged.passUnretained(self).toOpaque().debugDescription
    }
}
