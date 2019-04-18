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
    func getStyle(styleId: StyleId) -> Style;
    var totalLines: Int { get }
    var mode: EditViewController.Mode? { get }
}

/// A line-column index into a displayed text buffer.
typealias BufferPosition = (line: Int, column: Int)

let X_OFFSET: CGFloat = 2.0

class EditView: NSView {

    var lineSource: LineSource?

    override var isFlipped: Bool {
        return true
    }

    /// The smallest size, in measured points, that bounds the
    // entire document.
    var coreDocumentSize = CGSize.zero {
        didSet {
            if coreDocumentSize != oldValue {
                invalidateIntrinsicContentSize()
            }
        }
    }

    override var intrinsicContentSize: NSSize {
        let charSpace = DefaultFont.shared.characterWidth() * 2
        let lineHeight = DefaultFont.shared.linespace
        return CGSize(width: coreDocumentSize.width + charSpace,
                      height: coreDocumentSize.height + lineHeight)
    }

    override func draw(_ dirtyRect: NSRect) {
        guard let lines = lineSource, lines.totalLines > 0 else { return }
        NSColor.white.setFill()
        dirtyRect.fill()

        let font = DefaultFont.shared
        let linespace = font.linespace
        let charWidth = font.characterWidth()

        let first = min(Int((dirtyRect.minY / linespace).rounded(.down)), lines.totalLines)
        let last = min(Int((dirtyRect.maxY / linespace).rounded(.up)), lines.totalLines)

        for lineNumber in first..<last {
            let line = lines.getLine(line: UInt32(lineNumber)) ?? RawLine.placeholder()
            let attrString = NSMutableAttributedString(string: line.text, attributes: [.font: font, .foregroundColor: NSColor.black])

            for styleSpan in line.styles {
                let range = NSRange(location: styleSpan.start, length: styleSpan.len)
                let style = lines.getStyle(styleId: styleSpan.styleId)
                attrString.addAttributesForStyle(range, style: style)
            }

            let selY = font.topPadding + linespace * CGFloat(lineNumber)

            if let selection = line.selection {

                let selStart = font.isFixedPitch ? CGFloat(selection.startIndex) * charWidth : getVisualOffset(attrString, selection.startIndex)
                let selEnd = font.isFixedPitch ?  CGFloat(selection.endIndex) * charWidth : getVisualOffset(attrString, selection.endIndex)


                let rect = CGRect(x: X_OFFSET + selStart, y: selY, width: selEnd - selStart, height: linespace).integral
                NSColor.selectedTextBackgroundColor.setFill()
                rect.fill()
            }
            if let cursor = line.cursor {
                let cursorPos = getVisualOffset(attrString, cursor)

                let rect: NSRect
                if lines.mode?.drawBox ?? false {
                    let selWidth: CGFloat;
                    if font.isFixedPitch || cursorPos == 0 {
                        selWidth = charWidth
                    } else {
                        selWidth = cursorPos - getVisualOffset(attrString, cursor - 1)
                    }
                    rect = NSRect(x: X_OFFSET + max(cursorPos - selWidth, 0), y: selY, width: selWidth, height: linespace).integral
                    NSColor.lightGray.setFill()
                } else {
                    rect = NSRect(x: X_OFFSET + cursorPos, y: selY + (linespace - 1), width: charWidth, height: font.underlineThickness).integral
                    NSColor.black.setFill()
                }

                rect.fill()
            }
            let yPos = linespace * CGFloat(lineNumber)

            attrString.draw(at: NSPoint(x: X_OFFSET, y: yPos))

        }
    }

    func getVisualOffset(_ line: NSAttributedString, _ utf8Offset: Int) -> CGFloat {
        let index = line.string.utf16OffsetForUtf8Offset(utf8Offset)
        let ctLine = CTLineCreateWithAttributedString(line)
        let pos = CTLineGetOffsetForStringIndex(ctLine, index, nil)
        return pos
    }

    func yOffsetToLine(_ y: CGFloat) -> Int {
        let adjustY = max(y - DefaultFont.shared.topPadding, 0)
        return Int(floor(adjustY / DefaultFont.shared.linespace))
    }

    func lineIxToBaseline(_ lineIx: Int) -> CGFloat {
        return CGFloat(lineIx + 1) * DefaultFont.shared.linespace
    }

    func bufferPositionFromPoint(_ point: NSPoint) -> BufferPosition {
        let point = self.convert(point, from: nil)
        let lineIx = yOffsetToLine(point.y)
        if let line = lineSource?.getLine(line: UInt32(lineIx)) {
            let s = line.text
            let attrString = NSAttributedString(string: s, attributes: [.font: DefaultFont.shared])
            let ctline = CTLineCreateWithAttributedString(attrString)
            let relPos = NSPoint(x: point.x - X_OFFSET, y: lineIxToBaseline(lineIx) - point.y)
            let utf16_ix = CTLineGetStringIndexForPosition(ctline, relPos)
            if utf16_ix != kCFNotFound {
                let col = s.utf8offsetForUtf16Offset(utf16_ix)
                return BufferPosition(line: lineIx, column: col)
            }
        }
        return BufferPosition(line: lineIx, column: 0)
    }
}

//NOTE: this is imperfect. See https://stackoverflow.com/a/5635981
extension NSFont {
    var descent: CGFloat {
        return (-self.descender).rounded()
    }

    var linespace: CGFloat {
        return ceil(self.ascender.rounded() + descent + self.leading.rounded())
    }

    var topPadding: CGFloat {
        return descent + self.leading.rounded()
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
    //FIXME: figure out how to do actual bold & italics
    func italic() -> NSFont {
//        if let family = self.familyName {
//            let descriptor = fontDescriptor.withFamily(family).withSymbolicTraits(.italic)
//            return NSFont(descriptor: descriptor, size: 0) ?? self
//        }
        return self
    }

    func bold() -> NSFont {
//        if let family = self.familyName {
//            let descriptor = fontDescriptor.withFamily(family).withSymbolicTraits(.bold)
//            return NSFont(descriptor: descriptor, size: 0) ?? self
//        }
        return self
    }
}

extension String {
    func utf16OffsetForUtf8Offset(_ offsetUtf8: Int) -> Int {
        return self.utf8.index(self.utf8.startIndex, offsetBy: offsetUtf8).utf16Offset(in: self)
    }

    func utf8offsetForUtf16Offset(_ offsetUtf16: Int) -> Int {
        return Substring(self.utf16.prefix(offsetUtf16)).utf8.count
    }
}

extension NSMutableAttributedString {
    func addAttributesForStyle(_ range: NSRange, style: Style) {
        let start = self.string.utf16OffsetForUtf8Offset(range.location)
        let end = self.string.utf16OffsetForUtf8Offset(range.location + range.length)
        let utf16Range = NSRange(location: start, length: end - start)

        var attrs = [NSAttributedString.Key : Any]()
        if style.foreground.alphaComponent != 0 {
            attrs[.foregroundColor] = style.foreground
        }

        //FIXME: background color is always set, plus is paints over cursors.
        // And... probably selections. We should probably just handle this separately.
//        if style.background.alphaComponent != 0 {
//            attrs[.backgroundColor] = style.background
//        }

        if style.underline {
            attrs[.underlineStyle] = NSUnderlineStyle.single
        }

        if style.italic {
            attrs[.font] = DefaultFont.shared.italic()
        }

        if style.bold {
            attrs[.font] = DefaultFont.shared.bold()
        }
        self.addAttributes(attrs, range: utf16Range)
    }
}
