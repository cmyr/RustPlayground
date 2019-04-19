//
//  OutputViewController.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-04-16.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

class OutputViewController: NSViewController {
    let outputFont = DefaultFont.shared

    @IBOutlet var outputTextView: NSTextView!

    override func viewDidLoad() {
        super.viewDidLoad()
        outputTextView.frame = view.frame
    }

    func clearOutput() {
        self.outputTextView.string = ""
    }

    private func appendString(_ string: String, attributes: [NSAttributedString.Key : Any]?) {
        let EOFRange = NSRange(location: self.outputTextView.textStorage?.length ?? 0, length: 0)
        self.outputTextView.textStorage!.replaceCharacters(in: EOFRange, with: string)
        if EOFRange.location == 0 {
            self.outputTextView.font = self.outputFont
        }

        let attributes = attributes ?? [.foregroundColor: NSColor.black]
        let insertedRange = NSRange(location: EOFRange.location, length: string.count)
        self.outputTextView.textStorage?.addAttributes(attributes, range: insertedRange)

        let bottom = CGRect(x: 2, y: outputTextView.frame.maxY + 20, width: 0, height: 0)
        outputTextView.scrollToVisible(bottom)
    }
}

extension OutputViewController: RunnerOutputHandler {
    func printInfo(text: String) {
        let infoString = "[ \(text) ]\n"
        // hack to give stdout time to flush before printing 'done'
        DispatchQueue.main.asyncAfter(deadline: .now() + .milliseconds(5)) {
            self.appendString(infoString, attributes: [.foregroundColor: NSColor.systemGray])
        }
    }

    func handleStdOut(text: String) {
        DispatchQueue.main.async {
            self.appendString(text, attributes: nil)
        }
    }

    func handleStdErr(text: String) {
        DispatchQueue.main.async {
            self.appendString(text, attributes: nil)
        }
    }
}
