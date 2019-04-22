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
    static let stashDocumentKey = "net.cmyr.rust-playground.allDocumentContents"

    /// Convenience access to the AppDelegate instance
    static var shared: AppDelegate {
        return (NSApp.delegate as! AppDelegate)
    }

    let core = XiCoreProxy(rpcCallback: handleRpc, updateCallback: handleUpdate, widthMeasure: measureWidth)

    let styleMap = StyleMap()

    var mainController: EditViewController? {
        didSet {
            mainController?.core = core
        }
    }

    lazy var preferencesWindowController: PreferencesWindowController = {
        return NSStoryboard.main?.instantiateController(withIdentifier: "preferences") as! PreferencesWindowController;
    }()

    var scheduledEvents = [UInt32: NSEvent]()
    var nextWorkItemId: UInt32 = 0

    func applicationDidFinishLaunching(_ aNotification: Notification) {
        // uncomment me for vim mode
//        core.registerEventHandler(callback: dispatchEvent, action: handleRpc, timer: handleTimer, cancelTimer: cancelTimer)
        EditorPreferences.shared.syncAllWithCore()
        insertPlaceholderText()
        mainController?.view.window?.makeFirstResponder(mainController)
        DispatchQueue.global(qos: .default).async { [weak self] in
            let toolchains = listToolchains()
            DispatchQueue.main.async {
                self?.gotToolchains(toolchains)
            }
        }
    }

    func applicationWillTerminate(_ aNotification: Notification) {
        let bufferContents = core.getDocument()
        UserDefaults.standard.set(bufferContents, forKey: AppDelegate.stashDocumentKey)
    }

    func gotToolchains(_ toolchainResult: Result<[Toolchain], PlaygroundError>) {
        print("got toolchains \(toolchainResult)")
    }

    func insertPlaceholderText() {
        let placeholderProgram = "fn main() {\n    println!(\"hello 🦀!\");\n}"
        let savedContents = UserDefaults.standard.string(forKey: AppDelegate.stashDocumentKey)
        core.insertText(savedContents ?? placeholderProgram)
    }

    @IBAction func displayPreferencePane(_: Any?) {
        self.preferencesWindowController.showWindow(nil)
    }

    func handleMessage(method: String, params: [String: AnyObject]) {
        switch method {
        case "mode_change":
            let new_mode = params["mode"] as! String
            mainController?.mode = EditViewController.Mode(rawValue: new_mode)!
        case "parse_state":
            let newState = params["state"] as! String
            mainController?.parseState = newState;
        case "selector":
            let selector = params["sel"] as! String
            NSApp.sendAction(NSSelectorFromString(selector), to: nil, from: nil)
        case "content_size":
            let width = params["width"] as! Int
            let height = params["height"] as! Int
            mainController?.textLayoutSizeChanged(CGSize(width: width, height: height))
        case "scroll_to":
            let line = params["line"] as! Int
            let col = params["col"] as! Int
            mainController?.scrollTo(line, col: col)
        case "new_styles":
            // styles is a vec of (number, object) pairs
            let rawStyles = params["styles"] as! [[AnyObject]]
            rawStyles.map {
                let styleId = $0[0] as! UInt32
                let styleObject = $0[1] as! [String: AnyObject]
                return (styleId, Style.fromJson(styleObject))
                }
                .forEach { styleMap.addStyle(withId: $0, style: $1) }
        case "set_pasteboard":
            let text = params["text"] as! String
            let pasteboard = NSPasteboard.general
            pasteboard.clearContents()
            pasteboard.writeObjects([text as NSPasteboardWriting])
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
    if let ptr = jsonPtr {
        let string = String(cString: ptr)

        let message = try! JSONSerialization.jsonObject(with: string.data(using: .utf8)!) as! [String: AnyObject]
        let method = message["method"] as! String
        let params = message["params"] as! [String: AnyObject]

        (NSApp.delegate as! AppDelegate).handleMessage(method: method, params: params)
    }
}

func measureWidth(strPtr: UnsafePointer<Int8>?) -> XiSize {
    guard let strPtr = strPtr else {
        fatalError("measureWidth passed null pointer")
    }

    let string = String(cString: strPtr)
    let bounds = measureStringWidth(string)

    return XiSize(width: Int(bounds.width), height: Int(bounds.height))
}

func measureStringWidth(_ string: String, font: NSFont? = nil) -> CGRect {
    let font = font ?? EditorPreferences.shared.editorFont
    let attrString = NSAttributedString(string: string, attributes: [.font: font])

    let ctLine = CTLineCreateWithAttributedString(attrString)
    // FIXME: we only use height as a standin for linespace, so just swap
    // that in here. In the future we should have a separate method for getting
    // font info.
    let height = font.linespace
    let rect = CTLineGetBoundsWithOptions(ctLine, [])
    return CGRect(origin: rect.origin, size: CGSize(width: rect.width, height: height))
}
