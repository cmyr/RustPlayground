//
//  ViewController.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-03-18.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

class ViewController: NSViewController {

    @IBOutlet weak var editView: EditView!
    @IBOutlet weak var scrollView: NSScrollView!

    var contents = String();

    var core: XiCoreProxy!

    enum Mode: String {
        case command, insert
    }

    var totalLines: Int = 0;

    var parseState: String = "" {
        didSet {
//            self.stateLabel.stringValue = parseState
        }
    }

    var mode: Mode = .insert {
        didSet {
//            self.modeLabel.stringValue = mode.rawValue.capitalized(with: nil)
        }
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        NotificationCenter.default.addObserver(self,
                                               selector: #selector(frameDidChangeNotification),
                                               name: NSView.frameDidChangeNotification,
                                               object: scrollView)
        (NSApp.delegate as! AppDelegate).mainController = self
    }

    override func viewDidAppear() {
        self.mode = .insert
        self.editView.lineSource = self
        self.view.window?.makeFirstResponder(self)

    }

    func coreViewDidChange(core: XiCoreProxy, newLines: UInt32) {
        self.totalLines = Int(newLines)
        self.editView.needsDisplay = true
    }

    var contentSize: CGSize = CGSize.zero {
        didSet {
            if contentSize != oldValue {
            updateContentSize()
            }
        }
    }

    @objc func frameDidChangeNotification(_ notification: Notification) {
        core.frameChanged(newFrame: view.frame)
        updateContentSize()
    }

    func updateContentSize() {
        let cursorPadding = (DefaultFont.shared.characterWidth() * 3).rounded(.down)
        let size = CGSize(
            width: max(contentSize.width + cursorPadding, scrollView.contentSize.width),
            height: max(contentSize.height, scrollView.contentSize.height)
        )
        if size != editView.bounds.size {
            self.editView.frame = NSRect(origin: CGPoint.zero, size: size).integral
        }
    }

    override func doCommand(by selector: Selector) {
        let selString = NSStringFromSelector(selector)
        core.doCommand(selString)
    }

    override func insertText(_ insertString: Any) {
        core.insertText(insertString as! String)
        super.insertText(insertString)
    }

    override func keyDown(with event: NSEvent) {
        self.interpretKeyEvents([event])
    }
}

extension ViewController: LineSource {
    func getLine(line: UInt32) -> RawLine? {
        return core?.getLine(line)
    }
}
