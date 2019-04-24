//
//  MissingRustupInfoView.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-04-24.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

class MissingRustupInfoView: NSView {
    let labelView = NSTextView(frame: NSRect.zero)

    override var isOpaque: Bool {
        return true
    }

    override var isFlipped: Bool {
        return true
    }

    required init?(coder decoder: NSCoder) {
        super.init(coder: decoder)
        setup()
    }

    override init(frame frameRect: NSRect) {
        super.init(frame: frameRect)
        setup()
    }

    override func draw(_ dirtyRect: NSRect) {
        NSColor.white.setFill()
        dirtyRect.fill()
        
    }

    func setup() {
        labelView.translatesAutoresizingMaskIntoConstraints = false

        self.addSubview(labelView)
        addConstraints([
            NSLayoutConstraint(item: self, attribute: .centerX, relatedBy: .equal, toItem: labelView, attribute: .centerX, multiplier: 1, constant: 0),
            NSLayoutConstraint(item: self, attribute: .centerY, relatedBy: .equal, toItem: labelView, attribute: .centerY, multiplier: 1, constant: 0),
            NSLayoutConstraint(item: self, attribute: .width, relatedBy: .equal, toItem: labelView, attribute: .width, multiplier: 1, constant: 40),
            //FIXME: I really couldn't figure out how to just center this vertically, while respecting its intrinsicContentSize
            NSLayoutConstraint(item: labelView, attribute: .height, relatedBy: .greaterThanOrEqual, toItem: nil, attribute: .notAnAttribute, multiplier: 1, constant: 200),
            ])


        let text = "Oops! The Rust Playground requires rustup to manage rust toolchains.\n\nYou can install it at https://rustup.rs.";
        let attributes: [NSAttributedString.Key : Any] = [
            .font: NSFont.boldSystemFont(ofSize: 32.0),
            .foregroundColor: NSColor.systemGray
        ]
        let attrString = NSMutableAttributedString(string: text, attributes: attributes)
        let range = NSRange(location: 0, length: 0)
        labelView.textStorage!.replaceCharacters(in: range, with: attrString)

        labelView.isAutomaticLinkDetectionEnabled = true
        labelView.checkTextInDocument(nil)
        labelView.alignment = .center
        labelView.isEditable = false
        // required if we want link detection to work
        labelView.isSelectable = true
        self.needsLayout = true
    }
}
