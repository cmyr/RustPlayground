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

    var lineSource: LineSource?

    override var isFlipped: Bool {
        return true
    }

    override func draw(_ dirtyRect: NSRect) {
        guard let lines = lineSource, lines.totalLines > 0 else { return }
        NSColor.white.setFill()
        dirtyRect.fill()

        let font = DefaultFont.shared
        let linespace = font.linespace
        let xOff: CGFloat = 2.0
        let yOff = font.topPadding
        let charWidth = font.characterWidth()


        for lineNumber in 0..<lines.totalLines {
            let line = lines.getLine(line: UInt32(lineNumber))!
            let attrString = NSMutableAttributedString(string: line.text, attributes: [.font: font, .foregroundColor: NSColor.black])
            let yPos = yOff + linespace * CGFloat(lineNumber)
            if let selection = line.selection {

                let selStart = font.isFixedPitch ? CGFloat(selection.startIndex) * charWidth : getVisualOffset(attrString, selection.startIndex)
                let selEnd = font.isFixedPitch ?  CGFloat(selection.endIndex) * charWidth : getVisualOffset(attrString, selection.endIndex)
                let rect = CGRect(x: xOff + selStart, y: yPos, width: selEnd - selStart, height: linespace)
                NSColor.selectedTextBackgroundColor.setFill()
                rect.fill()
            }
            if let cursor = line.cursor {
                let cursorPos = font.isFixedPitch ? CGFloat(cursor) * charWidth : getVisualOffset(attrString, cursor)

                let rect = NSRect(x: xOff + cursorPos, y: yPos + (linespace - 1), width: charWidth, height: 1)
                NSColor.black.setFill()
                rect.fill()
            }

            attrString.draw(at: NSPoint(x: xOff, y: yPos))

        }
    }

    func getVisualOffset(_ line: NSAttributedString, _ cursorPos: Int) -> CGFloat {
        let ctLine = CTLineCreateWithAttributedString(line)
        let pos = CTLineGetOffsetForStringIndex(ctLine, cursorPos, nil)
        return pos
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
            let characters = [UniChar(0x20)]
            var glyphs = [CGGlyph(0)]
            if CTFontGetGlyphsForCharacters(self, characters, &glyphs, 1) {
                let advance = CTFontGetAdvancesForGlyphs(self, .horizontal, glyphs, nil, 1)
                return CGFloat(advance)
            }
        fatalError("font characterWidth() failed")
    }
}
