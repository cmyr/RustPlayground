//
//  ViewController.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-03-18.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

class ViewController: NSViewController {

    @IBOutlet weak var modeLabel: NSTextField!
    @IBOutlet weak var stateLabel: NSTextField!
    @IBOutlet weak var editView: EditView!

    var contents = String();

    var core: XiCoreProxy!

    enum Mode: String {
        case command, insert
    }

    var totalLines: Int = 0;

    var parseState: String = "" {
        didSet {
            self.stateLabel.stringValue = parseState
        }
    }

    var mode: Mode = .insert {
        didSet {
            self.modeLabel.stringValue = mode.rawValue.capitalized(with: nil)
        }
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        NotificationCenter.default.addObserver(self,
                                               selector: #selector(frameDidChangeNotification),
                                               name: NSView.frameDidChangeNotification,
                                               object: view)
        (NSApp.delegate as! AppDelegate).mainController = self
    }

    override func viewDidAppear() {
        self.mode = .insert
        self.editView.lineSource = self

    }

    func coreViewDidChange(core: XiCoreProxy, newLines: UInt32) {
        self.totalLines = Int(newLines)
        self.editView.needsDisplay = true
    }

    override func doCommand(by selector: Selector) {
        let selString = NSStringFromSelector(selector)
        (NSApp.delegate as! AppDelegate).core.doCommand(selString)
    }

    override func insertText(_ insertString: Any) {
        (NSApp.delegate as! AppDelegate).core.insertText(insertString as! String)
        super.insertText(insertString)
    }

    override func keyDown(with event: NSEvent) {
        self.interpretKeyEvents([event])
    }

    @objc func frameDidChangeNotification(_ notification: Notification) {
//        updateEditViewHeight()
//        willScroll(to: scrollView.contentView.bounds.origin)
//        updateViewportSize()
//        statusBar.checkItemsFitFor(windowWidth: self.view.frame.width)
    }
}

extension ViewController: LineSource {
    func getLine(line: UInt32) -> RawLine? {
        return core?.getLine(line)
    }
}
