//
//  ViewController.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-03-18.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

class EditViewController: NSViewController {

    @IBOutlet weak var editView: EditView!
    @IBOutlet weak var scrollView: NSScrollView!
    @IBOutlet weak var modeLabel: NSTextFieldCell!

    var contents = String()
    let minimumPadding: CGFloat = 2

    var core: XiCoreProxy!

    enum Mode: String {
        case command, insert, visual

        var drawBox: Bool {
            switch self {
            case .command, .visual:
                return true
            case .insert:
                return false
            }
        }
    }

    var totalLines: Int = 1;

    var parseState: String = "" {
        didSet {
            if let mode = self.mode {
                self.modeLabel.stringValue = "\(mode.rawValue.capitalized) \(parseState)"
            }
        }
    }

    var mode: Mode? = nil {
        didSet {
            if let mode = mode {
                self.modeLabel.controlView?.isHidden = false
                self.modeLabel.stringValue = mode.rawValue.capitalized(with: nil)
                self.editView.needsDisplay = true
            } else {
                self.modeLabel.controlView?.isHidden = true
            }
        }
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        NotificationCenter.default.addObserver(self,
                                               selector: #selector(frameDidChangeNotification),
                                               name: NSView.frameDidChangeNotification,
                                               object: scrollView)
        (NSApp.delegate as! AppDelegate).mainController = self
        modeLabel.font = DefaultFont.shared
        modeLabel.textColor = NSColor.lightGray
        if core.hasInputHandler {
            mode = .insert
        } else {
            mode = nil
        }
    }

    override func viewWillAppear() {
        super.viewWillAppear()
        view.window!.styleMask = [view.window!.styleMask, .fullSizeContentView]
        view.window!.titleVisibility = .hidden
    }

    override func viewDidAppear() {
        self.editView.lineSource = self
        self.view.window?.makeFirstResponder(self)
        updateCoreFrame()
    }

    func coreViewDidChange(core: XiCoreProxy, newLines: UInt32) {
        self.totalLines = Int(newLines)
        self.editView.setNeedsDisplay(scrollView.documentVisibleRect)
    }

    /// Called by core. `newSize` is the smallest size that bounds the entire
    /// document, in points.
    func textLayoutSizeChanged(_ newSize: CGSize) {
        editView.coreDocumentSize = newSize
        view.needsLayout = true
    }

    func scrollTo(_ line: Int, col: Int) {
        let y = CGFloat(line) * DefaultFont.shared.linespace + minimumPadding
        let lineText = core.getLine(UInt32(line))!
        let toMeasure = lineText.text.utf8.prefix(col)
        let x = measureStringWidth(String(toMeasure)!).width

        // one line is the current line, one line is padding
        let height = DefaultFont.shared.linespace * 2
        let width = DefaultFont.shared.characterWidth() * 2
        let rect = CGRect(origin: CGPoint(x: x, y: y),
                          size: CGSize(width: width , height: height)).integral

            editView.scrollToVisible(rect)
            editView.setNeedsDisplay(editView.visibleRect)
    }

    @objc func frameDidChangeNotification(_ notification: Notification) {
        updateCoreFrame()
    }

    private var coreFrame = CGRect.zero {
        didSet {
            if coreFrame != oldValue {
                core.frameChanged(newFrame: coreFrame)
            }
        }
    }

    /// Send the current frame to core. This is used for determining the visible
    /// region, and for word wrapping.
    func updateCoreFrame() {
        let docFrame = scrollView.documentVisibleRect
        let cursorPadding = DefaultFont.shared.characterWidth() + minimumPadding * 2
        let size = CGSize(width: max(docFrame.width - cursorPadding, 0), height: docFrame.height)
        //FIXME: 'ensureNonZero' is a hack, figure out how to do content insets
        coreFrame = CGRect(origin: docFrame.origin.ensureNonZero(), size: size)
    }

    override func doCommand(by selector: Selector) {
        let selString = NSStringFromSelector(selector)
        core.doCommand(selString)
    }

    override func insertText(_ insertString: Any) {
        core.insertText(insertString as! String)
    }

    override func keyDown(with event: NSEvent) {
        self.interpretKeyEvents([event])
    }

    // Determines the gesture type based on flags and click count.
    private func clickGestureType(event: NSEvent) -> Any {
        func granularity(for event: NSEvent) -> String {
            if event.clickCount >= 3 {
                return "line"
            } else if event.clickCount == 2 {
                return "word"
            } else {
                return "point"
            }
        }

        let gran = granularity(for: event)

        if event.modifierFlags.contains(.shift) {
            return [
                "select_extend": [
                    "granularity": gran
                ]
            ]
        } else {
            return [
                "select": [
                    "granularity": gran,
                    "multi": event.modifierFlags.contains(.command)
                ]
            ]
        }
    }

    private var lastDragPosition: BufferPosition?
    /// handles autoscrolling when a drag gesture exists the window
    private var dragTimer: Timer?
    private var dragEvent: NSEvent?

    override func mouseDown(with theEvent: NSEvent) {

        let position = editView.bufferPositionFromPoint(theEvent.locationInWindow)
        lastDragPosition = position
        core.doGesture(position: position,
                       type: clickGestureType(event: theEvent))

        dragTimer = Timer.scheduledTimer(timeInterval: TimeInterval(1.0/60), target: self, selector: #selector(_autoscrollTimerCallback), userInfo: nil, repeats: true)
        dragEvent = theEvent
    }

    override func mouseDragged(with theEvent: NSEvent) {
        editView.autoscroll(with: theEvent)
        let position = editView .bufferPositionFromPoint(theEvent.locationInWindow)
        if let last = lastDragPosition, last != position {
            lastDragPosition = position
            core.doGesture(position: position, type: "drag")
        }
        dragEvent = theEvent
    }

    override func mouseUp(with theEvent: NSEvent) {
        dragTimer?.invalidate()
        dragTimer = nil
        dragEvent = nil
    }

    @objc func _autoscrollTimerCallback() {
        if let event = dragEvent {
            mouseDragged(with: event)
        }
    }

    @objc func paste(_ sender: AnyObject?) {
        print("paste")
        NSPasteboard
            .general
            .string(forType: .string)
            .flatMap({ core.insertText($0) })
    }

    override func selectAll(_ sender: Any?) {
        core.sendRpc(method: "selectAll:", params: [])
    }

    @objc func undo(_ sender: AnyObject?) {
        core.doCommand("undo")
    }

    @objc func redo(_ sender: AnyObject?) {
        core.doCommand("redo")
    }

    @objc func cut(_ sender: AnyObject?) {
        core.doCommand("cut")
    }

    @objc func copy(_ sender: AnyObject?) {
        core.doCommand("copy")
    }

    override func indent(_ sender: Any?) {
        core.sendRpc(method: "indent", params: [])
    }

    @objc func unindent(_ sender: Any?) {
        core.sendRpc(method: "outdent", params: [])
    }

    @objc func reindent(_ sender: Any?) {
        core.sendRpc(method: "reindent", params: [])
    }
}

extension EditViewController: LineSource {
    func getLine(line: UInt32) -> RawLine? {
        return core?.getLine(line)
    }

    func getStyle(styleId: StyleId) -> Style {
        return (NSApp.delegate as! AppDelegate).styleMap.style(forId: styleId)
    }
}

extension NSPoint {
    func ensureNonZero() -> NSPoint {
        return NSPoint(x: max(0, self.x), y: max(0, self.y))
    }
}
