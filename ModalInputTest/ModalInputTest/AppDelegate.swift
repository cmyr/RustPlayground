//
//  AppDelegate.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-03-18.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

@NSApplicationMain
class AppDelegate: NSObject, NSApplicationDelegate {
    let core = XiCoreProxy(rpcCallback: handleRpc, updateCallback: handleUpdate, widthMeasure: measureWidth)

    var mainController: ViewController? {
        didSet {
            mainController?.core = core
        }
    }
    var scheduledEvents = [UInt32: NSEvent]()
    var nextWorkItemId: UInt32 = 0

    func applicationDidFinishLaunching(_ aNotification: Notification) {
        // uncomment me for vim mode
//        core.registerEventHandler(callback: dispatchEvent, action: handleAction, timer: handleTimer, cancelTimer: cancelTimer)
    }

    func applicationWillTerminate(_ aNotification: Notification) {
        // Insert code here to tear down your application
    }

    func handleMessage(method: String, params: [String: AnyObject]) {
        switch method {
        case "mode_change":
            let new_mode = params["mode"] as! String
            mainController?.mode = ViewController.Mode(rawValue: new_mode)!
        case "parse_state":
            let newState = params["state"] as! String
            mainController?.parseState = newState;
        case "selector":
            let selector = params["sel"] as! String
            NSApp.sendAction(NSSelectorFromString(selector), to: nil, from: nil)
        default:
            print("unhandled method \(method)")
        }
    }

    func sendEvent(_ event: NSEvent) {
        // first send any currently delayed events
        self.scheduledEvents.keys.forEach { self.sendScheduledEvent(withIdentifier: $0) }
        if let window = event.window as? XiWindow {
            window.reallySendEvent(event)
        }
    }

    func scheduleEvent(_ event: NSEvent, afterDelay delayMillis: UInt32) -> UInt32 {
        let nextId = self.nextWorkItemId;
        self.nextWorkItemId += 1;

        let workItem = DispatchWorkItem {
            self.sendScheduledEvent(withIdentifier: nextId)
        }

        self.scheduledEvents[nextId] = event
        DispatchQueue.main.asyncAfter(deadline: .now() + .milliseconds(Int(delayMillis)), execute: workItem)
        return nextId
    }

    func sendScheduledEvent(withIdentifier ident: UInt32) {
        if let event = self.scheduledEvents[ident] {
            if let window = event.window as? XiWindow {
                print("sending delayed event \(ident)")
                window.reallySendEvent(event)
                self.core.clearPending(ident)
            }
        }
        self.scheduledEvents.removeValue(forKey: ident)
    }

    func cancelEvent(withIdentifier ident: UInt32) {
        self.scheduledEvents.removeValue(forKey: ident)
    }

    func coreDidUpate(_ invalRange: Range<Int>) {
        mainController?.coreViewDidChange(core: core, newLines: UInt32(invalRange.count))
    }
}

func dispatchEvent(eventPtr: OpaquePointer?, toTheTrash: Bool) {
    if let ptr = UnsafeRawPointer(eventPtr) {
        let event: NSEvent = Unmanaged<NSEvent>.fromOpaque(ptr).takeRetainedValue();
        if !toTheTrash {
            (NSApp.delegate as! AppDelegate).sendEvent(event)
        }
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

func handleTimer(eventPtr: OpaquePointer?, delay: UInt32) -> UInt32 {
    let event: NSEvent = Unmanaged<NSEvent>.fromOpaque(UnsafeRawPointer(eventPtr)!).takeRetainedValue();
    return (NSApp.delegate as! AppDelegate).scheduleEvent(event, afterDelay: delay)
}

func cancelTimer(token: UInt32) {
    (NSApp.delegate as! AppDelegate).cancelEvent(withIdentifier: token)
}

func handleUpdate(start: Int, end: Int) {
    (NSApp.delegate as! AppDelegate).coreDidUpate(start..<end)
}

func handleRpc(jsonPtr: UnsafePointer<Int8>?) {

}

class DefaultFont {
    static let shared = NSFont(name: "Input Sans", size: 14.0)!
}

func measureWidth(strPtr: UnsafePointer<Int8>?) -> XiSize {
    guard let strPtr = strPtr else {
        fatalError("measureWidth passed null pointer")
    }

    let string = String(cString: strPtr)
    let attrString = NSAttributedString(string: string, attributes: [.font: DefaultFont.shared])

    let ctLine = CTLineCreateWithAttributedString(attrString)
    let bounds = CTLineGetBoundsWithOptions(ctLine, [])
    return XiSize(width: Int(bounds.width), height: Int(bounds.height))
}
