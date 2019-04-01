//
//  EditView.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-03-30.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

protocol LineSource {
    func getLine(line: UInt32) -> RawLine?;
    var totalLines: Int { get }
}

class EditView: NSView {

    let defaultFont = NSFont(name: "Menlo", size: 14.0)!
    var lineSource: LineSource?

    override var isFlipped: Bool {
        return true
    }

    override func draw(_ dirtyRect: NSRect) {
        guard let lines = lineSource, lines.totalLines > 0 else { return }
        NSColor.white.setFill()
        dirtyRect.fill()
        let linespace = defaultFont.linespace
        let xOff: CGFloat = 2.0
        let yOff = defaultFont.topPadding
        let charWidth = defaultFont.characterWidth()

        for lineNumber in 0..<lines.totalLines {
            let line = lines.getLine(line: UInt32(lineNumber))!
            print("line \(lineNumber):", line.text)
            let attrString = NSMutableAttributedString(string: line.text, attributes: [.font: defaultFont, .foregroundColor: NSColor.black])
            let yPos = yOff + linespace * CGFloat(lineNumber)
            if let selection = line.selection {

                let selStart = CGFloat(selection.startIndex)
                let selEnd = CGFloat(selection.endIndex)
                print("selection \(selStart)..\(selEnd)")
                let rect = CGRect(x: xOff + selStart * charWidth, y: yPos, width: charWidth * (selEnd - selStart), height: linespace)
                NSColor.selectedTextColor.setFill()
                rect.fill()
            }
            if let cursor = line.cursor {
                print("cursor \(cursor)")
                let cursorPos = CGFloat(cursor)
                let rect = NSRect(x: xOff + cursorPos * charWidth, y: yPos, width: charWidth, height: linespace)
                NSColor.lightGray.setFill()
                rect.fill()
            }

            attrString.draw(at: NSPoint(x: xOff, y: yPos))

        }
    }
    
}

extension NSFont {
    var descent: CGFloat {
        return -self.descender
    }

    var linespace: CGFloat {
        return ceil(self.ascender + descent + self.leading)
    }

    var topPadding: CGFloat {
        return descent + self.leading
    }

    func characterWidth() -> CGFloat {
        if self.isFixedPitch {
            let characters = [UniChar(0x20)]
            var glyphs = [CGGlyph(0)]
            if CTFontGetGlyphsForCharacters(self, characters, &glyphs, 1) {
                let advance = CTFontGetAdvancesForGlyphs(self, .horizontal, glyphs, nil, 1)
                return CGFloat(advance)
            }
        }
        return 0
    }
}
