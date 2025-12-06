#!/bin/bash
# Verify all Cantarell NURBS glyphs have correct properties

set -e

NURBS_DIR="chi_fonts/nurbie/Cantarell-VF/glyphs"

echo "Verifying Cantarell NURBS glyphs..."

total=0
passed=0

for file in "$NURBS_DIR"/*.nurbs.json; do
    total=$((total + 1))

    # Check each contour has degree 3, non-null control points, and proper knots
    degree=$(jq '.contours[0].degree' "$file")
    weights=$(jq '.contours[0].weights' "$file")
    num_cp=$(jq '.contours[0].control_points | length' "$file")
    num_knots=$(jq '.contours[0].knots | length' "$file")
    first_knot=$(jq '.contours[0].knots[0]' "$file")
    last_knot=$(jq '.contours[0].knots[-1]' "$file")

    if [ "$degree" != "3" ]; then
        echo "❌ $file: degree=$degree (expected 3)"
        continue
    fi

    if [ "$weights" != "null" ]; then
        echo "❌ $file: weights=$weights (expected null)"
        continue
    fi

    expected_knots=$((num_cp + 4))
    if [ "$num_knots" != "$expected_knots" ]; then
        echo "❌ $file: knots=$num_knots (expected $expected_knots for $num_cp control points)"
        continue
    fi

    if [ "$first_knot" != "0" ]; then
        echo "❌ $file: first knot=$first_knot (expected 0)"
        continue
    fi

    if [ "$last_knot" != "1" ]; then
        echo "❌ $file: last knot=$last_knot (expected 1)"
        continue
    fi

    passed=$((passed + 1))
done

echo ""
echo "Results: $passed/$total glyphs passed validation"

if [ "$passed" -eq "$total" ]; then
    echo "✅ All glyphs valid!"
    exit 0
else
    echo "❌ Some glyphs failed validation"
    exit 1
fi
